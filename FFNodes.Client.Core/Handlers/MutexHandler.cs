/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using System.IO.Pipes;

namespace FFNodes.Client.Core.Handlers;

public static class MutexHandler
{
    public static void HandleMutex(string name)
    {
        _ = new Mutex(true, name, out bool created);
        if (!created)
        {
            SendArgumentsToExistingInstance(name);
            Environment.Exit(0); // exit the application.
        }
        else
        {
            CommandLineHandler.HandleCommandLine(Environment.GetCommandLineArgs()[1..]);
            StartNamedPipeServer(name);
        }
    }

    private static void SendArgumentsToExistingInstance(string name)
    {
        string[] args = Environment.GetCommandLineArgs()[1..]; // get the command line arguments, excluding the first argument which is the executable path.
        using NamedPipeClientStream clientPipe = new(".", name, PipeDirection.Out); // a client pipe stream is used to send messages.
        try
        {
            clientPipe.Connect(); // attempts to connect to the server.
            using StreamWriter writer = new(clientPipe); // a stream writer is used to write messages to the server.
            writer.Write(string.Join(" ", args)); // join the arguments into a single string and send it to the server.
            writer.Flush(); // flush the stream writer to send the message.
        }
        catch (Exception ex)
        {
            Console.WriteLine("Error sending arguments to existing instance: " + ex.Message); // shit when wrong!?!?!?!
        }
    }

    private static void StartNamedPipeServer(string name)
    {
        Task.Run(() => // Creates a new thread to not lock the main thread.
        {
            while (true) // Creating an infinite loop to keep the server alive.
            {
                Console.WriteLine("Waiting for arguments!");
                using NamedPipeServerStream serverPipe = new(name, PipeDirection.In); // a server pipe stream is used to receive messages.
                try
                {
                    serverPipe.WaitForConnection(); // this method blocks the thread until a client connects.
                    using (StreamReader reader = new(serverPipe))
                    {
                        CommandLineHandler.HandleCommandLine(reader.ReadToEnd().Split(' '));
                    }
                    serverPipe.Disconnect(); // disconnect the client once the message is received.
                }
                catch
                {
                    // This always throws an exception when receiving a message. I don't know why.
                    // Ignore this exception.
                }
            }
        });
    }
}