/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

using FFNodes.Core.Model;
using FFNodes.Server.Data;
using Newtonsoft.Json;
using Timer = System.Timers.Timer;

namespace FFNodes.Server.Handlers;

public class UserHandler
{
    public static readonly UserHandler Instance = Instance ??= new UserHandler();
    private readonly List<User> users;
    private readonly Dictionary<User, Timer> connectedUsers;

    private UserHandler()
    {
        users = new();
        connectedUsers = new();
    }

    /// <summary>
    /// Adds the user to the list of users if the user does not already exist.
    /// </summary>
    /// <param name="user"></param>
    /// <returns>if the user was successfully added.</returns>
    public bool CreateUser(User user)
    {
        // Checks if the user is already in the list of users.
        if (!users.Any(u => u.Id.Equals(user.Id) || u.Username.Equals(user.Username)))
        {
            if (!users.Any())
            {
                user.IsAdmin = true;
            }
            // If not, add them.
            users.Add(user);
            return true;
        }
        return false;
    }

    /// <summary>
    /// Pings a user to keep them connected.
    /// </summary>
    /// <param name="user"></param>
    public void PingUser(User user)
    {
        // Checks if the user is already connected, if so, stop the timer and dispose of it.
        if (connectedUsers.ContainsKey(user))
        {
            connectedUsers[user].Stop();
            connectedUsers[user].Dispose();
        }
        // Create a new timer for the user.
        Timer timer = new(TimeSpan.FromSeconds(30))
        {
            AutoReset = false,
            Enabled = true,
        };
        timer.Elapsed += (sender, e) =>
        {
            // if the user has not pinged the server in 30 seconds, remove them from the connected
            connectedUsers.Remove(user);
            users.Remove(user);
        };
        timer.Start();
        connectedUsers[user] = timer;
        user.LastOnline = DateTime.Now;
        Save(user);
        Load(user);
    }

    /// <summary>
    /// Gets a user by their id.
    /// </summary>
    /// <param name="id">the users id</param>
    /// <returns>a user or null if a user is not found.</returns>
    public User? GetUser(Guid id) => users.FirstOrDefault(x => x.Id.Equals(id));

    /// <summary>
    /// Gets a list of all the connected users.
    /// </summary>
    /// <returns></returns>
    public User[] GetConnectedUsers() => connectedUsers.Keys.ToArray();

    public void Load()
    {
        // Loops through all the files in the users directory and deserializes them.
        foreach (string file in Directory.GetFiles(Directories.Users, "*.json"))
        {
            // Opens the file and reads it.
            using StreamReader reader = File.OpenText(file);
            // Deserializes the user.
            User? user = JsonConvert.DeserializeObject<User>(reader.ReadToEnd());
            if (user != null)
            {
                // Adds the user to the list of users.
                users.Add(user);
            }
        }
    }

    /// <summary>
    /// Loads a user by the users id.
    /// </summary>
    /// <param name="user"></param>
    public void Load(Guid id)
    {
        // Opens the file and reads it.
        using StreamReader reader = File.OpenText(Path.Combine(Directories.Users, $"{id:N}.json"));

        // Deserializes the user.
        User? loadedUser = JsonConvert.DeserializeObject<User>(reader.ReadToEnd());
        if (loadedUser != null)
        {
            // Removes the user from the list of users if they already exist.
            User? item = users.FirstOrDefault(u => u.Id.Equals(loadedUser.Id));
            if (item != null)
            {
                users.Remove(item);
            }
            // Adds the user to the list of users.
            users.Add(loadedUser);
        }
    }

    /// <summary>
    /// Loads a user.
    /// </summary>
    /// <param name="user"></param>
    public void Load(User user) => Load(user.Id);

    /// <summary>
    /// Saves a user.
    /// </summary>
    /// <param name="user">The user that will be saved</param>
    public void Save(User user)
    {
        // Seralizes the user and writes it to a file, overwrites the file if it already exists.
        using StreamWriter writer = File.CreateText(Path.Combine(Directories.Users, $"{user.Id:N}.json"));
        writer.Write(JsonConvert.SerializeObject(user));
    }

    /// <summary>
    /// Saves all the users.
    /// </summary>
    public void SaveAll()
    {
        // Loops through all the users and saves them.
        foreach (User user in users)
        {
            Save(user);
        }
    }
}