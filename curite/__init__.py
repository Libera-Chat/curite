from asyncio import Future
from random import randint
from re import compile as re_compile
from typing import Dict

from ircrobots import Bot as BaseBot
from ircrobots import Server as BaseServer
from ircrobots.formatting import strip as format_strip
from irctokens import Line, build

RE_SUCCESS = re_compile(r"^(?P<account>\S+) has now been verified.$")
RE_ALREADY = re_compile(r"^(?P<account>\S+) is not awaiting verification.$")
RE_INVALID = re_compile(r"^verification failed. invalid key for (?P<account>\S+).$")
RE_ISNTREG = re_compile(r"^(?P<account>\S+) is not registered.$")


class CuriteServer(BaseServer):
    def __init__(self, bot: BaseBot, name: str):
        super().__init__(bot, name)
        self._waiting: Dict[str, Future[bool]] = {}
        self.nickserv_name = "NickServ"

    def verify(self, account: str, token: str):
        account = self.casefold(account)
        if account in self._waiting:
            future = self._waiting[account]
        else:
            future = self._waiting[account] = Future()
            self.send(
                build(
                    "PRIVMSG",
                    [self.nickserv_name, f"VERIFY REGISTER {account} {token}"],
                )
            )
        return future

    def _verify_result(self, account: str, result: bool):
        if (account := self.casefold(account)) in self._waiting:
            self._waiting.pop(account).set_result(result)

    async def line_read(self, line: Line):
        if line.command == "NOTICE" and self.casefold_equals(
            line.hostmask.nickname, "nickserv"
        ):

            message = format_strip(line.params[1])

            if (p_success := RE_SUCCESS.search(message)) is not None:
                self._verify_result(p_success.group("account"), True)

            elif (p_already := RE_ALREADY.search(message)) is not None:
                self._verify_result(p_already.group("account"), False)

            elif (p_invalid := RE_INVALID.search(message)) is not None:
                self._verify_result(p_invalid.group("account"), False)

            elif (p_isntreg := RE_ISNTREG.search(message)) is not None:
                self._verify_result(p_isntreg.group("account"), False)

    def line_preread(self, line: Line):
        print(f"< {line.format()}")

    def line_presend(self, line: Line):
        print(f"> {line.format()}")


class Bot(BaseBot):
    def create_server(self, name: str):
        return CuriteServer(self, name)
