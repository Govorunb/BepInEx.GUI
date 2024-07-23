using BepInEx.Logging;

namespace BepInEx.GUI.Loader;

internal static class Log
{
    private static readonly ManualLogSource _logSource = Logger.CreateLogSource("BepInEx.GUI.Loader"); 

    internal static void LogDebug(object data) => _logSource.LogDebug(data);
    internal static void LogError(object data) => _logSource.LogError(data);
    internal static void LogFatal(object data) => _logSource.LogFatal(data);
    internal static void LogInfo(object data) => _logSource.LogInfo(data);
    internal static void LogMessage(object data) => _logSource.LogMessage(data);
    internal static void LogWarning(object data) => _logSource.LogWarning(data);
}
