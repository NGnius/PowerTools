import logging
import json
import os
import pathlib

import asyncio
from aiohttp import web

HOME_DIR = str(pathlib.Path(os.getcwd()).parent.parent.resolve())
SETTINGS_DIR = HOME_DIR + "/.config/powertools"

if not os.path.exists(SETTINGS_DIR):
    os.mkdir(SETTINGS_DIR)

http_runner = None
http_server = None

class GameInfo:
    def __init__(self, gameid: int, game_info: dict):
        self.gameid = gameid
        self.game_info = game_info

    def appid(self):
        return self.game_info["appid"]

    def name(self):
        return self.game_info["display_name"]

    def settings_path(self) -> str:
        return SETTINGS_DIR + os.pathsep + str(self.appid()) + ".json"

    def load_settings(self) -> dict:
        settings_path = self.settings_path()
        if os.exists(settings_path):
            with open(settings_path, mode="r") as f:
                return json.load(f)
        return None

    def has_settings(self) -> bool:
        return os.exists(self.settings_path())


class Server(web.Application):

    def __init__(self, version):
        super().__init__()
        self.version = version
        self.current_game = None
        self.last_recognised_game = None
        self.add_routes([
            web.get("/", lambda req: self.index(req)),
            web.post("/on_game_start/{game_id}", lambda req: self.on_game_start(req)),
            web.post("/on_game_exit/{game_id}", lambda req: self.on_game_exit(req)),
            web.post("/on_game_exit_null", lambda req: self.on_game_exit_null(req))
        ])
        logging.debug("Server init complete")

    def game(self) -> GameInfo:
        return self.current_game

    def recognised_game(self) -> GameInfo:
        return self.last_recognised_game

    async def index(self, request):
        logging.debug("Debug index page accessed")
        return web.json_response({
            "name": "PowerTools",
            "version": self.version,
            "latest_game_id": self.current_game,
            "latest_recognised_game_id": self.last_recognised_game,
        }, headers={"Access-Control-Allow-Origin": "*"})

    async def on_game_start(self, request):
        game_id = request.match_info["game_id"]
        data = await request.text()
        logging.debug(f"on_game_start {game_id} body:\n{data}")
        try:
            game_id = int(game_id)
            data = json.loads(data)
        except:
            return web.Response(text="WTF", status=400)
        self.current_game = GameInfo(game_id, data)
        if True: # TODO check for game_id in existing profiles
            self.last_recognised_game = self.current_game # only set this when profile exists
            # TODO apply profile
        return web.Response(status=204, headers={"Access-Control-Allow-Origin": "*"})

    async def on_game_exit(self, request):
        # ignored for now
        game_id = request.match_info["game_id"]
        data = await request.text()
        logging.debug(f"on_game_exit {game_id}")
        try:
            game_id = int(game_id)
        except ValueError:
            return web.Response(text="WTF", status=400)
        if self.current_game.gameid == game_id:
            pass
            #self.current_game = None
            # TODO change settings to default
        return web.Response(status=204, headers={"Access-Control-Allow-Origin": "*"})

    async def on_game_exit_null(self, request):
        # ignored for now
        logging.info(f"on_game_exit_null")
        #self.current_game = None
        # TODO change settings to default
        return web.Response(status=204, headers={"Access-Control-Allow-Origin": "*"})

async def start(version):
    global http_runner, http_server
    loop = asyncio.get_event_loop()
    http_server = Server(version)
    http_runner = web.AppRunner(http_server)
    await http_runner.setup()
    site = web.TCPSite(http_runner, '0.0.0.0', 5030)
    await site.start()

async def shutdown(): # never really called
    global http_runner, http_server
    if http_runner is not None:
        await http_runner.cleanup()
        http_runner = None
    http_server = None
