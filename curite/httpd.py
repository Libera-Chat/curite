import asyncio, traceback
from argparse  import ArgumentParser
from asyncio   import StreamReader, StreamWriter
from typing    import List

from async_timeout import timeout as timeout_

from irctokens import build
from ircrobots import Bot
from ircrobots.matching   import Response, Nick, Folded, Formatless, SELF
from ircrobots.formatting import strip as format_strip

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
            # closed connection
            return

    return headers

NICKSERV = Nick("NickServ")
async def _verify(
        bot:     Bot,
        account: str,
        token:   str) -> bool:

    success = f"{account} has now been verified."

    server  = list(bot.servers.values())[0]
    async with server.read_lock:
        await server.send(build("PRIVMSG", ["NickServ", f"VERIFY REGISTER {account} {token}"]))
        verify_line = await server.wait_for({
            Response("NOTICE", [SELF, Formatless(Folded(f"{account} is not awaiting verification."))], source=NICKSERV),
            Response("NOTICE", [SELF, Formatless(Folded(f"verification failed. invalid key for {account}."))], source=NICKSERV),
            Response("NOTICE", [SELF, Formatless(Folded(f"{account} is not registered."))], source=NICKSERV),
            Response("NOTICE", [SELF, Formatless(Folded(success))], source=NICKSERV)
        })

        verify_msg = format_strip(verify_line.params[1])
        return server.casefold_equals(success, verify_msg)

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
            return

        method, path, _   = headers[0].decode("ascii").split(" ", 2)

        path_match = config.path_pattern.search(path)
        if not path_match:
            return

        account = path_match.group("account")
        token   = path_match.group("token")

        try:
            async with timeout_(5):
                verified = await _verify(bot, account, token)
        except asyncio.TimeoutError:
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
