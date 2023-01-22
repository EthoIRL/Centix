using frontend.Api;
using frontend.Config;
using frontend.Web;

namespace frontend;

public class Program
{
    public static ConfigManager ConfigManager;
    private static FrontendServer _frontend;
    public static ApiUtils ApiUtils = null!;
    
    public static void Main(string[] args)
    {
        ConfigManager = new ConfigManager(Environment.CurrentDirectory);
        new Thread(() => ApiUtils = new ApiUtils()).Start();
        new Thread(() => _frontend = new FrontendServer()).Start();
    }
}