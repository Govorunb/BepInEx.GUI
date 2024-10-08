﻿using System.IO;
using BepInEx.Configuration;

namespace BepInEx.GUI.Loader;

internal static class LoaderConfig
{
    internal const string FileName = "BepInEx.GUI.cfg";
    internal static string ConfigFilePath { get; private set; }

    private static ConfigFile File { get; set; }

    internal const string EnableBepInExGUIConfigKey = "Enable BepInEx GUI";
    internal const string EnableBepInExGUIConfigDescription = "Enable the custom BepInEx GUI";
    internal static ConfigEntry<bool> EnableBepInExGUIConfig { get; private set; }

    internal const string OpenEvenWhenBepInExConsoleIsEnabledConfigKey = "Open Even When BepInEx Console Is Enabled";
    internal const string OpenEvenWhenBepInExConsoleIsEnabledConfigDescription = "By default, the GUI does not launch if the BepInEx console is enabled";
    internal static ConfigEntry<bool> OpenEvenWhenBepInExConsoleIsEnabled { get; private set; }

    internal const string CloseWindowWhenGameLoadedConfigKey = "Close Window When Game Loaded";
    internal const string CloseWindowWhenGameLoadedConfigDescription = "Close the graphic user interface window when the game is loaded";
    internal static ConfigEntry<bool> CloseWindowWhenGameLoadedConfig { get; private set; }

    internal const string CloseWindowWhenGameClosesConfigKey = "Close Window When Game Closes";
    internal const string CloseWindowWhenGameClosesConfigDescription = "Close the graphic user interface window when the game closes";
    internal static ConfigEntry<bool> CloseWindowWhenGameClosesConfig { get; private set; }

    internal static void Init(string folderFullPath) => Init(new ConfigFile(Path.Combine(folderFullPath, FileName), true));

    internal static void Init(ConfigFile file)
    {
        ConfigFilePath = file.ConfigFilePath;
        File = file;
        EnableBepInExGUIConfig = File.Bind("Settings", EnableBepInExGUIConfigKey, true, EnableBepInExGUIConfigDescription);

        OpenEvenWhenBepInExConsoleIsEnabled = File.Bind("Settings", OpenEvenWhenBepInExConsoleIsEnabledConfigKey, false, OpenEvenWhenBepInExConsoleIsEnabledConfigDescription);

        CloseWindowWhenGameLoadedConfig = File.Bind("Settings", CloseWindowWhenGameLoadedConfigKey, false, CloseWindowWhenGameLoadedConfigDescription);

        CloseWindowWhenGameClosesConfig = File.Bind("Settings", CloseWindowWhenGameClosesConfigKey, true, CloseWindowWhenGameClosesConfigDescription);
    }
}
