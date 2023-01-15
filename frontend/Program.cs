using frontend.Web;

namespace frontend;

public class Program
{
    private static FrontendServer _frontend;
    
    public static void Main(string[] args)
    {
        new Thread(() => _frontend = new FrontendServer()).Start();
    }
}