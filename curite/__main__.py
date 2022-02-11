import asyncio
from argparse import ArgumentParser
from ircrobots import ConnectionParams

from . import Bot
from .config import Config, load as config_load
from .httpd import run as httpd_run


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
    await bot.add_server(host, params)
    await asyncio.wait([httpd_run(bot, config), bot.run()])


if __name__ == "__main__":
    parser = ArgumentParser()
    parser.add_argument("config")
    args = parser.parse_args()

    config = config_load(args.config)
    asyncio.run(main(config))
