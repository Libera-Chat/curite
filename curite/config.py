from dataclasses import dataclass
from lib2to3.pgen2 import token
from re import compile as re_compile
from typing import Pattern, Tuple

import yaml


@dataclass
class Config:
    server: str
    nickname: str
    password: str
    nickserv_name: str
    socket_path: str

    url_success: str
    url_failure: str

    path_pattern: Pattern


def load(filepath: str):
    with open(filepath) as file:
        config_yaml = yaml.safe_load(file.read())

    return Config(
        server=config_yaml["server"],
        nickname=config_yaml["nickname"],
        password=config_yaml["password"],
        nickserv_name=config_yaml["nickserv-name"],
        socket_path=config_yaml["socket-path"],
        url_success=config_yaml["url-success"],
        url_failure=config_yaml["url-failure"],
        path_pattern=re_compile(config_yaml["path-pattern"]),
    )
