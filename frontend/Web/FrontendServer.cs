using frontend.Analytics;
using frontend.Utils;
using Microsoft.AspNetCore.Mvc.Razor;
using Microsoft.Extensions.FileProviders;

namespace frontend.Web;

public class FrontendServer
{
    // TODO: Config
    public FrontendServer()
    {
        Console.WriteLine(Path.GetFullPath($"{Directory.GetCurrentDirectory()}{Path.DirectorySeparatorChar}Web{Path.DirectorySeparatorChar}"));
        
        var builder = WebApplication.CreateBuilder(new WebApplicationOptions
        {
            EnvironmentName = Environments.Production,
            ContentRootPath = Path.GetFullPath($"{Directory.GetCurrentDirectory()}{Path.DirectorySeparatorChar}Web{Path.DirectorySeparatorChar}"),
            WebRootPath = "assets"
        });
        
        builder.Services.AddControllers();
        
        builder.WebHost.UseKestrel(options =>
        {
            // options.ListenAnyIP(Program.ConfigManager.Config.FrontendPorts.http);
            // options.ListenAnyIP(Program.ConfigManager.Config.FrontendPorts.https, configure => configure.UseHttps());

            // int? fileUploadMax = Program.ConfigManager.Config.FileNetworkUploadMax;
            // if (fileUploadMax != null)
            // {
            //     fileUploadMax *= 100000000;
            // }
            //
            // options.Limits.MaxRequestBodySize = fileUploadMax;
        });
        
        builder.Services.AddRazorPages(options => { options.RootDirectory = "/Web/Pages"; });

        builder.Services.Configure<RazorViewEngineOptions>(configure =>
        {
            configure.ViewLocationExpanders.Add(new ViewLocationExpansion());
        });
        
        var app = builder.Build();

        if (!app.Environment.IsDevelopment())
        {
            app.UseExceptionHandler("/Error");
            app.UseHsts();
        }

        if (Program.ConfigManager.Config.AnalyticsApi != null)
        {
            app.UseAnalyticsMiddleware(Program.ConfigManager.Config.AnalyticsApi);
        }

        app.UseStaticFiles();
        
        Console.WriteLine($"Path: {app.Environment.ContentRootPath}");

        app.UseStaticFiles(new StaticFileOptions
        {
            FileProvider = new PhysicalFileProvider(
                Path.Combine(app.Environment.ContentRootPath, "Assets")),
            RequestPath = "/Web/Assets"
        });

        app.UseRouting();
        
        app.UseStatusCodePagesWithRedirects("/error/{0}");

        app.UseAuthorization();

        app.MapRazorPages();

        app.UseEndpoints(endpoints => { endpoints.MapControllers(); });

        app.Run();
    }
}