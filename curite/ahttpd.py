from __future__ import annotations

import asyncio
from typing import TYPE_CHECKING

from aiohttp import web
from async_timeout import timeout

if TYPE_CHECKING:
    from aiohttp.web import StreamResponse

    from curite import CuriteServer
    from curite.config import Config


class WebServer:
    def __init__(
        self,
        bot: CuriteServer,
        config: Config,
    ) -> None:
        self.bot = bot
        self.app = web.Application()

        self.app.add_routes(
            [
                web.post(
                    f"/{{account:{config.account_re}}}/{{token:{config.token_re}}}",
                    self.on_post,
                )
            ]
        )
        self.config = config

        self.runner = web.AppRunner(self.app)

        pass

    async def listen_and_serve(self):
        await self.runner.setup()
        listener: web.BaseSite
        if self.config.unix_socket_path != "":
            listener = web.UnixSite(self.runner, self.config.unix_socket_path)

        elif self.config.httpd_port > 0:
            listener = web.TCPSite(self.runner, "localhost", self.config.httpd_port)

        else:
            raise ValueError("No configured listener location")

        await listener.start()

    async def on_post(self, request: web.Request) -> StreamResponse:
        account = request.match_info.get("account")
        token = request.match_info.get("token")

        if account is None or token is None:
            print(f"! Bad request for {account=} and {token=}")
            raise web.HTTPBadRequest()

        try:
            async with timeout(5):
                verified = await self.bot.verify(account, token)
        except asyncio.TimeoutError:
            raise web.HTTPInternalServerError(
                reason="Unable to verify. Please try again later."
            )

        if verified:
            raise web.HTTPSeeOther(self.config.url_success)

        raise web.HTTPSeeOther(self.config.url_failure)
