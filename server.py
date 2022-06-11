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


class Server(web.Application):

    def __init__(self, version):
        super().__init__()
        self.version = version
        self.current_game = None
        self.add_routes([
            web.get("/", lambda req: self.index(req)),
            web.post("/on_game_start/{game_id}", lambda req: self.on_game_start(req)),
            web.post("/on_game_exit/{game_id}", lambda req: self.on_game_exit(req)),
            web.post("/on_game_exit_null", lambda req: self.on_game_exit_null(req)),
            web.get("/self_destruct", lambda req: self.self_destruct(req))
        ])
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

    async def index(self, request):
        logging.debug("Debug index page accessed")
        current_game = None if self.current_game is None else self.current_game.gameid
        game_info = None if self.current_game is None else self.current_game.game_info
        settings_info = None if self.current_game is None else self.current_game.load_settings()
        return web.json_response({
            "name": "PowerTools",
            "version": self.version,
            "latest_game_id": current_game,
            "game_info": game_info,
            "settings": settings_info
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
        if self.current_game.has_settings():
            self.last_recognised_game = self.current_game
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
            self.current_game = None
        return web.Response(status=204, headers={"Access-Control-Allow-Origin": "*"})

    async def on_game_exit_null(self, request):
        # ignored for now
        logging.info(f"on_game_exit_null")
        self.current_game = None
        return web.Response(status=204, headers={"Access-Control-Allow-Origin": "*"})

    async def self_destruct(self, request):
        logging.warning("Geodude self-destructed")
        await shutdown()
        # unreachable \/ \/ \/
        return web.Response(status=204, headers={"Access-Control-Allow-Origin": "*"})

async def start(version):
    global http_runner, http_server, http_site
    # make sure old server has shutdown
    try:
        async with aiohttp.ClientSession() as session:
            async with session.get('http://127.0.0.1:5030/self_destruct') as response:
                await response.text()
    except:
        pass
    http_server = Server(version)
    http_runner = web.AppRunner(http_server)
    await http_runner.setup()
    site = web.TCPSite(http_runner, '127.0.0.1', 5030)
    await site.start()

async def shutdown(): # never really called
    global http_runner, http_server, http_site
    if http_runner is not None:
        await http_runner.cleanup()
        http_runner = None
        http_site.stop()
    http_site = None
    http_server = None
