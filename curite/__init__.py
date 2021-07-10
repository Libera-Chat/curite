from random    import randint
from irctokens import Line
from ircrobots import Bot as BaseBot
from ircrobots import Server as BaseServer

class Server(BaseServer):
    async def handshake(self):
        nick_prefix = f"{self.params.username}-"
        remaining   = 16-len(nick_prefix)
        random_max  = (10**remaining)-1
        random      = str(randint(0, random_max)).zfill(remaining)
        self.params.nickname = f"{nick_prefix}{random}"
        await super().handshake()

    def line_preread(self, line: Line):
        print(f"< {line.format()}")
    def line_presend(self, line: Line):
        print(f"> {line.format()}")

class Bot(BaseBot):
    def create_server(self, name: str):
        return Server(self, name)

