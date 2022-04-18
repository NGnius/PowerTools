class Plugin:
    # A normal method. It can be called from JavaScript using call_plugin_function("method_1", argument1, argument2)
    async def method_1(self, *args):
        pass

    # A normal method. It can be called from JavaScript using call_plugin_function("method_2", argument1, argument2)
    async def method_2(self, *args):
        pass

    # Asyncio-compatible long-running code, executed in a task when the plugin is loaded
    async def _main(self):
        pass