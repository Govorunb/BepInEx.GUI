global using System;
global using System.Collections.Generic;
global using System.Text;
using System.Diagnostics;
using System.IO;
using System.Net;
using System.Net.Sockets;
using BepInEx.Configuration;
using System.Reflection;
using BepInEx.Logging;
using Mono.Cecil;

namespace BepInEx.GUI.Loader;

internal partial class EntryPoint
{
    public static IEnumerable<string> TargetDLLs { get; } = Array.Empty<string>();

    private static EntryPoint Instance = new();

    public static void Patch(AssemblyDefinition _) { }

#if BEPINEX_5
    public static void Initialize() => Instance.Init();
#endif
    private void Init()
    {
        try
        {
            InitializeInternal();
        }
        catch (Exception e)
        {
            Log.LogError($"Failed to initialize : ({e.GetType()}) {e.Message}{Environment.NewLine}{e}");
        }
    }

    private void InitializeInternal()
    {
#if BEPINEX_5
        LoaderConfig.Init(Paths.ConfigPath);
#else
        LoaderConfig.Init(Config);
#endif

        var consoleConfig = (ConfigEntry<bool>)typeof(BepInPlugin).Assembly.
            GetType("BepInEx.ConsoleManager", true).
            GetField("ConfigConsoleEnabled",
            BindingFlags.Static | BindingFlags.Public).GetValue(null);
        if (consoleConfig.Value && !LoaderConfig.OpenEvenWhenBepInExConsoleIsEnabled.Value)
        {
            Log.LogInfo("BepInEx regular console is enabled, aborting launch.");
        }
        else if (LoaderConfig.EnableBepInExGUIConfig.Value)
        {
            FindAndLaunchGUI();
        }
        else
        {
            Log.LogInfo("Custom BepInEx.GUI is disabled in the config, aborting launch.");
        }
    }

    private string FindGUIExecutable()
    {
        foreach (var filePath in Directory.GetFiles(Paths.PatcherPluginPath, "*", SearchOption.AllDirectories))
        {
            var fileName = Path.GetFileName(filePath);

            const string GuiFileName = "bepinex_gui";

            // No platform check because proton is used for RoR2 and it handles it perfectly anyway:
            // It makes the Process.Start still goes through proton and makes the bep gui
            // that was compiled for Windows works fine even in linux operating systems.

            if (fileName == $"{GuiFileName}.exe")
            {
                var versInfo = FileVersionInfo.GetVersionInfo(filePath);
                if (versInfo.FileMajorPart == 3)
                {
                    Log.LogInfo($"Found bepinex_gui v3 executable in {filePath}");
                    return filePath;
                }
            }
        }

        return null;
    }

    private void FindAndLaunchGUI()
    {
        Log.LogInfo("Finding and launching GUI");

        var executablePath = FindGUIExecutable();
        if (executablePath != null)
        {
            var freePort = FindFreePort();
            var process = LaunchGUI(executablePath, freePort);
            if (process != null)
            {
                Logger.Listeners.Add(new SendLogToClientSocket(freePort));
                Logger.Listeners.Add(new CloseProcessOnChainloaderDone(process));
            }
            else
            {
                Log.LogInfo("LaunchGUI failed");
            }
        }
        else
        {
            Log.LogInfo("bepinex_gui executable not found.");
        }
    }

    private static int FindFreePort()
    {
        int port = 0;
        Socket socket = new(AddressFamily.InterNetwork, SocketType.Stream, ProtocolType.Tcp);
        try
        {
            IPEndPoint localEP = new(IPAddress.Any, 0);
            socket.Bind(localEP);
            localEP = (IPEndPoint)socket.LocalEndPoint;
            port = localEP.Port;
        }
        finally
        {
            socket.Close();
        }

        return port;
    }

    private static Process LaunchGUI(string executablePath, int socketPort)
    {
        var processStartInfo = new ProcessStartInfo();
        processStartInfo.FileName = executablePath;
        processStartInfo.WorkingDirectory = Path.GetDirectoryName(executablePath);

        processStartInfo.Arguments =
            $"\"{typeof(Paths).Assembly.GetName().Version}\" " +
            $"\"{Paths.ProcessName}\" " +
            $"\"{Paths.GameRootPath}\" " +
            $"\"{GetLogOutputFilePath()}\" " +
            $"\"{LoaderConfig.ConfigFilePath}\" " +
            $"\"{Process.GetCurrentProcess().Id}\" " +
            $"\"{socketPort}\"";

        return Process.Start(processStartInfo);
    }

    // Bad and hacky way to retrieve the correct log file path
    private static string GetLogOutputFilePath()
    {
        foreach (var logListener in Logger.Listeners)
        {
            if (logListener is DiskLogListener diskLogListener)
            {
                return ((diskLogListener.LogWriter as StreamWriter)?.BaseStream as FileStream)?.Name
                    ?? Path.Combine(Paths.BepInExRootPath, "LogOutput.log");
            }
        }

        return "";
    }
}
