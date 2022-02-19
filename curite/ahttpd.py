from __future__ import annotations

import asyncio
from typing import cast, TYPE_CHECKING

from aiohttp import web
from async_timeout import timeout

from curite import CuriteServer

if TYPE_CHECKING:
    from aiohttp.web import StreamResponse

    from curite.config import Config

    from ircrobots import Bot


class VerifyError(Exception):
    pass


class WebServer:
    def __init__(
        self,
        bot: Bot,
        config: Config,
    ) -> None:
        self._bot = bot
        self._config = config

        self._app = web.Application()
        self._app.add_routes([web.post("/{path:.*}", self._post)])
        self._runner = web.AppRunner(self._app)

    async def run_noblock(self):
        await self._runner.setup()
        print("AAAA")
        listener = web.UnixSite(self._runner, self._config.socket_path)
        await listener.start()

    async def _verify(self, path: str) -> None:
        match = self._config.path_pattern.search(path)
        if match is None:
            raise VerifyError("invalid path")

        servers = list(self._bot.servers.values())
        if not servers:
            raise VerifyError("server unavailable")

        server = cast(CuriteServer, servers[0])
        account = match.group("account")
        token = match.group("token")
        try:
            async with timeout(5):
                verified = await server.verify(account, token)
        except asyncio.TimeoutError:
            raise VerifyError("timeout")

        if not verified:
            raise VerifyError("nickserv disagrees")

    async def _post(self, request: web.Request) -> StreamResponse:
        try:
            await self._verify(request.path)
        except VerifyError:
            raise web.HTTPSeeOther(self._config.url_failure)
        else:
            raise web.HTTPSeeOther(self._config.url_success)
