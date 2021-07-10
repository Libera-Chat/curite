from dataclasses import dataclass
from re          import compile as re_compile
from typing      import Pattern, Tuple

import yaml

@dataclass
class Config(object):
    server:     Tuple[str, int, bool]
    nickname:   str
    password:   str
    httpd_port: int

    url_success: str
    url_failure: str

    path_pattern: Pattern

def load(filepath: str):
    with open(filepath) as file:
        config_yaml = yaml.safe_load(file.read())

    nickname = config_yaml["nickname"]
    server   = config_yaml["server"]
    hostname, port_s = server.split(":", 1)
    tls      = False

    if port_s.startswith("+"):
        tls    = True
        port_s = port_s.lstrip("+")
    port = int(port_s)


    return Config(
        (hostname, port, tls),
        nickname,
        config_yaml["password"],
        config_yaml["httpd-port"],
        config_yaml["url-success"],
        config_yaml["url-failure"],
        re_compile(config_yaml["path-pattern"])
    )
