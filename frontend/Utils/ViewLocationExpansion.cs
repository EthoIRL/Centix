using Microsoft.AspNetCore.Mvc.Razor;

namespace frontend.Utils;

public class ViewLocationExpansion : IViewLocationExpander
{
    public void PopulateValues(ViewLocationExpanderContext context) {}

    public IEnumerable<string> ExpandViewLocations(ViewLocationExpanderContext context, IEnumerable<string> viewLocations)
    {
        return new[]
        {
            "/Web/Pages/{1}/{0}.cshtml",
            "/Web/Pages/{0}.cshtml",
            "/Web/Pages/Account/{0}.cshtml",
            "/Web/Pages/Codes/{0}.cshtml"
        };
    }
}