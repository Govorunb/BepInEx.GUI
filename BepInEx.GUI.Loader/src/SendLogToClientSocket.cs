using System.Net;
using System.Net.Sockets;
using System.Threading;
using BepInEx.Logging;

namespace BepInEx.GUI.Loader;

internal class SendLogToClientSocket : ILogListener
{
    private int _freePort;

    private readonly Thread _thread;

    private readonly object _queueLock = new();
    private readonly Queue<LogEventArgs> _logQueue = new();

    private bool _isDisposed = false;

    internal static SendLogToClientSocket Instance { get; private set; }
    public LogLevel LogLevelFilter { get; } = LogLevel.All;

    internal SendLogToClientSocket(int freePort)
    {
        Instance = this;

        _freePort = freePort;

        _thread = new Thread(() =>
        {
            var ipAddress = IPAddress.Parse("127.0.0.1");

            var listener = new TcpListener(ipAddress, _freePort);

            listener.Start();

            while (true)
            {
                Log.LogInfo($"[SendLogToClient] Accepting Socket.");
                var clientSocket = listener.AcceptSocket();

                if (_isDisposed)
                {
                    break;
                }

                SendPacketsToClientUntilConnectionIsClosed(clientSocket);
            }
        });

        _thread.Start();
    }

    private void SendPacketsToClientUntilConnectionIsClosed(Socket clientSocket)
    {
        while (true)
        {
            if (_isDisposed)
            {
                break;
            }

            while (_logQueue.Count > 0)
            {
                LogEventArgs log;
                lock (_queueLock)
                {
                    log = _logQueue.Peek();
                }
                var logPacket = new LogPacket(log);

                try
                {
                    clientSocket.Send(logPacket.Bytes);
                }
                catch (Exception e)
                {
                    Log.LogError($"Error while trying to send log to socket: {e}{Environment.NewLine}Disconnecting socket.");
                    return;
                }

                lock (_queueLock)
                {
                    _ = _logQueue.Dequeue();
                }
            }
        }
    }

    public void Dispose()
    {

    }

    internal void StoreLog(LogEventArgs eventArgs)
    {
        lock (_queueLock)
        {
            _logQueue.Enqueue(eventArgs);
        }
    }

    public void LogEvent(object sender, LogEventArgs eventArgs)
    {
        if (_isDisposed)
        {
            return;
        }

        StoreLog(eventArgs);
    }
}
