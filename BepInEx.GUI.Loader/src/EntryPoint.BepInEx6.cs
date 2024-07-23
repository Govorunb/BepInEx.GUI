#if BEPINEX_6
using BepInEx.Preloader.Core.Patching;

namespace BepInEx.GUI.Loader;

[PatcherPluginInfo("BepInEx.GUI.Loader", "BepInEx.GUI.Loader", "3.0.3-Govorunb")]
partial class EntryPoint : BasePatcher
{
    public override void Initialize()
    {
        base.Initialize();
        Init();
    }
}
#endif