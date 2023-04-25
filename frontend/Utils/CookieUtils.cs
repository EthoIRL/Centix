using frontend.Api;
using frontend.Api.Models.User;

namespace frontend.Utils;

public static class CookieUtils
{
    public const string CookieName = "session_key";

    public static async Task<bool> IsCookieValid(ApiUtils apiUtils, HttpRequest? request, HttpResponse response)
    {
        if (!IsCookieReal(request, response))
        {
            return false;
        }

        // Check authenticity server-side
        var userCredentials = new ModelUserKey
        {
            key = request.Cookies[CookieName]
        };

        var userInfo =
            await apiUtils.PostAndReceiveModel<ModelUserKey, ModelUserInfo>(
                Program.ConfigManager.Config.BackendApiUri + "/user/info", userCredentials);
        if (userInfo != null)
        {
            return ResetCookieExpire(request, response);
        }

        return false;
    }

    public static async Task<ModelUserInfo?> IsCookieUserValid(ApiUtils apiUtils, HttpRequest? request,
        HttpResponse response)
    {
        if (!IsCookieReal(request, response))
        {
            return null;
        }

        // Check authenticity server-side
        var userCredentials = new ModelUserKey
        {
            key = request.Cookies[CookieName]
        };

        var userInfo =
            await apiUtils.PostAndReceiveModel<ModelUserKey, ModelUserInfo>(
                Program.ConfigManager.Config.BackendApiUri + "/user/info", userCredentials);
        
        if (userInfo != null)
        {
            if (ResetCookieExpire(request, response))
            {
                return userInfo;
            }
        }

        return null;
    }

    private static bool ResetCookieExpire(HttpRequest? request, HttpResponse response)
    {
        if (IsCookieReal(request, response))
        {
            response.Cookies.Delete(CookieName);
            response.Cookies.Append(CookieName, request.Cookies[CookieName]!, new CookieOptions
            {
                IsEssential = true,
                Secure = true,
                SameSite = SameSiteMode.Lax,
                Expires = DateTimeOffset.Now.AddDays(30)
            });
            return true;
        }

        return false;
    }

    private static bool IsCookieReal(HttpRequest? request, HttpResponse response)
    {
        if (request == null || request?.Cookies == null)
        {
            return false;
        }
        
        // if auth cookie does not exist
        if (request.Cookies[CookieName] == null)
        {
            return false;
        }

        // if auth cookie has an empty value
        if (request.Cookies[CookieName] == String.Empty)
        {
            response.Cookies.Delete(CookieName);
            return false;
        }

        // if auth cookie is longer than expected
        if (request.Cookies[CookieName]?.Length > 48)
        {
            response.Cookies.Delete(CookieName);
            return false;
        }

        return true;
    }
}