import asyncio
from argparse import ArgumentParser

from ircrobots import ConnectionParams

from curite import ahttpd

from . import Bot, CuriteServer
from .config import Config
from .config import load as config_load


async def main(config: Config):
    bot = Bot()

    host, port, tls = config.server
    params = ConnectionParams(
        config.nickname,
        host,
        port,
        tls,
        realname=config.nickname,
        password=config.password,
    )

    cs: CuriteServer = await bot.add_server(host, params)  # type: ignore

    cs.nickserv_name = config.nickserv_name

    webserver = ahttpd.WebServer(cs, config)
    await webserver.listen_and_serve()
    await bot.run()
    await webserver.runner.cleanup()


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("config")
    args = parser.parse_args()

    config = config_load(args.config)
    asyncio.run(main(config))
