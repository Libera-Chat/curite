import asyncio, traceback
from argparse     import ArgumentParser
from asyncio      import StreamReader, StreamWriter
from typing       import cast, List
from urllib.parse import unquote as url_unquote

from async_timeout import timeout as timeout_

from ircrobots import Bot

from . import CuriteServer
from .config import Config

async def _headers(
        reader: StreamReader
        ) -> List[str]:
    headers: List[bytes] = []
    buffer = b""

    while True:
        data = await reader.read(2048)
        if data:
            buffer += data
            headers.extend(buffer.split(b"\r\n"))
            buffer  = headers.pop(-1)
            if b"" in headers:
                break
        else:
            raise ConnectionError("client closed connection")

    return headers

async def run(
        bot:    Bot,
        config: Config):

    async def _client(
            reader: StreamReader,
            writer: StreamWriter
            ):

        try:
            async with timeout_(10):
                headers = await _headers(reader)
        except asyncio.TimeoutError:
            print("! header timeout")
            return
        except ConnectionError as e:
            print(f"! header error {str(e)}")
            return

        method, path, _   = headers[0].decode("ascii").split(" ", 2)
        if not method == "POST":
            return

        path_match = config.path_pattern.search(path)
        if not path_match:
            return

        account = url_unquote(path_match.group("account"))
        token   = path_match.group("token")

        if not bot.servers:
            return
        server = cast(CuriteServer, list(bot.servers.values())[0])

        try:
            async with timeout_(5):
                verified = await server.verify(account, token)
        except asyncio.TimeoutError:
            print("! verify timeout")
            return

        if verified:
            url = config.url_success
        else:
            url = config.url_failure

        data = "\r\n".join([
            "HTTP/1.1 302 Moved",
            f"Location: {url}"
        ]).encode("utf8")
        # HTTP headers end with an empty line
        data += b"\r\n\r\n"

        try:
            async with timeout_(5):
                writer.write(data)
                await writer.drain()
                writer.close()
                await writer.wait_closed()
        except Exception as e:
            traceback.print_exc()
            return

    server = await asyncio.start_server(_client, "", config.httpd_port)
    async with server:
        await server.serve_forever()
