import logging
import json
import os
import pathlib

# import asyncio
from aiohttp import web
import aiohttp

HOME_DIR = "/home/deck"
SETTINGS_DIR = HOME_DIR + "/.config/powertools"

if not os.path.exists(SETTINGS_DIR):
    os.mkdir(SETTINGS_DIR)

http_runner = None
http_server = None
http_site = None

class GameInfo:
    def __init__(self, gameid: int, game_info: dict):
        self.gameid = gameid
        self.game_info = game_info

    def appid(self):
        return self.game_info["appid"]

    def name(self):
        return self.game_info["display_name"]

    def settings_path(self) -> str:
        return SETTINGS_DIR + os.path.sep + str(self.appid()) + ".json"

    def load_settings(self) -> dict:
        settings_path = self.settings_path()
        if os.path.exists(settings_path):
            with open(settings_path, mode="r") as f:
                return json.load(f)
        return None

    def has_settings(self) -> bool:
        return os.path.exists(self.settings_path())


class Server:

    def __init__(self, version):
        super().__init__()
        self.version = version
        self.current_game = None
        logging.debug("Server init complete")

    def game(self) -> GameInfo:
        return self.current_game

    def set_game(self, game_id, data):
        self.current_game = GameInfo(game_id, data)

    def unset_game(self, game_id):
        if self.current_game is None:
            return
        if game_id is None or self.current_game.gameid == game_id:
            self.current_game = None

async def start(version):
    global http_server
    http_server = Server(version)

async def shutdown(): # never really called
    global http_server
    http_server = None
