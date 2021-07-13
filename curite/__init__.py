from random    import randint
from irctokens import Line
from ircrobots import Bot as BaseBot
from ircrobots import Server as BaseServer

class Server(BaseServer):
    async def handshake(self):
        nickname = self.params.realname
        while "#" in nickname:
            onerand  = str(randint(0, 9))
            nickname = nickname.replace("#", onerand, 1)

        self.params.nickname = nickname
        self.params.username = nickname
        await super().handshake()

    def line_preread(self, line: Line):
        print(f"< {line.format()}")
    def line_presend(self, line: Line):
        print(f"> {line.format()}")

class Bot(BaseBot):
    def create_server(self, name: str):
        return Server(self, name)

