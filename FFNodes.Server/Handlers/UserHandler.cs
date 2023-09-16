/*
    FFNodes - LFInteractive LLC. 2021-2024
    FFNodes is a client/server solution for batch processing ffmpeg operations from multiple systems across the internet.
    Licensed under GPL-3.0
    https://www.gnu.org/licenses/gpl-3.0.en.html#license-text
*/

// Ignore Spelling: username

using FFNodes.Core.Data;
using FFNodes.Core.Model;
using FFNodes.Server.Data;
using Microsoft.Extensions.Primitives;
using System.Diagnostics.CodeAnalysis;
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

            // Adds the users id to the configuration file.
            List<Guid> items = Configuration.Instance.Users.ToList();
            items.Add(user.Id);
            Configuration.Instance.Users = items.ToArray();
            Configuration.Instance.Save();

            Save(user);
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
            FileSystemHandler.Instance.MarkUsersActiveProcessesAsAbandoned(user);
        };
        timer.Start();
        connectedUsers[user] = timer;

        // Update the users last online time.
        DateTime now = DateTime.Now;
        if (user.LastOnline < now - TimeSpan.FromMinutes(1))
        {
            user.ActiveTime += now - user.LastOnline;
        }
        user.LastOnline = now;

        // Reloads the user
        Save(user);
        Load(user);
    }

    /// <summary>
    /// Attempts to get a user by their id.
    /// </summary>
    /// <param name="id"></param>
    /// <param name="user"></param>
    /// <returns></returns>
    public bool TryGetUser(Guid id, [NotNullWhen(true)] out User user) => (user = GetUser(id)) != null;

    /// <summary>
    /// Gets a user by their id.
    /// </summary>
    /// <param name="id">the users id</param>
    /// <returns>a user or null if a user is not found.</returns>
    public User? GetUser(Guid id) => users.FirstOrDefault(x => x.Id.Equals(id));

    /// <summary>
    /// Gets a user by their username.
    /// </summary>
    /// <param name="username"></param>
    /// <returns></returns>
    public User? GetUser(string username) => users.FirstOrDefault(x => x.Username.Equals(username));

    /// <summary>
    /// Gets a list of all the connected users.
    /// </summary>
    /// <returns></returns>
    public User[] GetConnectedUsers() => connectedUsers.Keys.ToArray();

    /// <summary>
    /// Gets a list of all the users.
    /// </summary>
    /// <returns></returns>
    public User[] GetUsers() => users.ToArray();

    /// <summary>
    /// Loads all the users from the users directory.
    /// </summary>
    public void Load()
    {
        // Loops through all the files in the users directory and deserializes them.
        foreach (Guid userId in Configuration.Instance.Users)
        {
            Load(userId);
        }
    }

    /// <summary>
    /// Attempts to get a user from the headers of a request.
    /// </summary>
    /// <param name="context"></param>
    /// <param name="user"></param>
    /// <returns></returns>
    public bool GetUserFromHeaders(HttpContext context, out User user)
    {
        user = null; // Sets the user to null by default.
        // Checks if the user id header exists, if so, attempts to parse it.
        return context.Request.Headers.TryGetValue("User-ID", out StringValues user_id)
           && user_id.Any()
           && Guid.TryParse(user_id[0], out Guid id)
           && TryGetUser(id, out user)
           && user != null;
    }

    /// <summary>
    /// Loads a user by the users id.
    /// </summary>
    /// <param name="user"></param>
    public void Load(Guid id)
    {
        using DatabaseFile usersDatabase = DatabaseFile.Open(Path.Combine(Files.UsersDatabase));
        // Loads the user from the users directory.
        User? loadedUser = usersDatabase.ReadEntry<User>(id);
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
        using DatabaseFile usersDatabase = DatabaseFile.Open(Path.Combine(Files.UsersDatabase));
        // Serializes the user and writes it to a file, overwrites the file if it already exists.
        usersDatabase.WriteEntry(user.Id, user);
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

    /// <summary>
    /// Gets the users database.
    /// </summary>
    /// <returns></returns>
    public DatabaseFile GetDatabase() => DatabaseFile.Open(Path.Combine(Files.UsersDatabase));
}