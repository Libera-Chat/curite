import asyncio
from argparse import ArgumentParser

from ircrobots import ConnectionParams

from curite import ahttpd

from . import Bot, CuriteServer
from .config import Config
from .config import load as config_load


async def main(config: Config):
    bot = Bot()

    params = ConnectionParams.from_hoststring(config.nickname, config.server)
    await bot.add_server("server", params)

    webserver = ahttpd.WebServer(bot, config)
    await webserver.run_noblock()
    await bot.run()


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("config")
    args = parser.parse_args()

    config = config_load(args.config)
    asyncio.run(main(config))
