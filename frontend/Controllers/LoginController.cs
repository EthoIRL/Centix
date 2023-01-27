using System.Net;
using frontend.Api.Models.User;
using frontend.Api.Models.User.Login;
using frontend.Models.Account;
using frontend.Utils;
using Microsoft.AspNetCore.Mvc;

namespace frontend.Controllers;

[ApiController]
public class LoginController : Controller
{
    [HttpPost("/login/")]
    public async Task<ActionResult> Login([FromForm] LoginModel model)
    {
        if (ModelState.IsValid)
        {
            var userCredentials = new ModelUserCredentials
            {
                username = model.username,
                password = model.password
            };

            var response =
                await Program.ApiUtils.PostAndReceiveResponse(
                    Program.ConfigManager.Config.BackendApiUri + "/user/login", userCredentials);

            if (response?.StatusCode == HttpStatusCode.OK)
            {
                var loginResponse = await response.Content.ReadFromJsonAsync<ModelLoginResponse>();

                if (loginResponse != null && loginResponse.key != String.Empty)
                {
                    if (Request.Cookies[CookieUtils.CookieName] != null)
                    {
                        Response.Cookies.Delete(CookieUtils.CookieName);
                    }

                    Response.Cookies.Append(CookieUtils.CookieName, loginResponse.key, new CookieOptions
                    {
                        IsEssential = true,
                        Secure = true,
                        SameSite = SameSiteMode.Lax,
                        Expires = DateTimeOffset.Now.AddDays(30)
                    });

                    return Redirect("~/account");
                }
            }
            else if (response != null)
            {
                var error = await response.Content.ReadFromJsonAsync<ModelError>();
                ModelState.AddModelError("Error",
                    error != null ? error.error : "An internal error has occured within the server!");
            }
        }

        if (ModelState.ErrorCount == 0)
        {
            ModelState.AddModelError("Error", "An internal error has occured within the server!");
        }

        return View("~/Web/Pages/Account/Login.cshtml", model);
    }
}