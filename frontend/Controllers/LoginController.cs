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

            var response = await Program.ApiUtils.PostAndReceiveModel
                    <ModelUserCredentials, ModelLoginResponse>(Program.ConfigManager.Config.BackendApiUri + "/user/login", userCredentials);

            if (response != null && response.key != String.Empty)
            {
                if (Request.Cookies[CookieUtils.CookieName] != null)
                {
                    Response.Cookies.Delete(CookieUtils.CookieName);
                }
            
                Response.Cookies.Append(CookieUtils.CookieName, response.key, new CookieOptions
                {
                    IsEssential = true,
                    Secure = true,
                    SameSite = SameSiteMode.Lax,
                    Expires = DateTimeOffset.Now.AddDays(30)
                });

                return Redirect("~/account");
            }
        }
        
        ModelState.AddModelError("Error", "Invalid username or password! Please try again.");
        return View("~/Web/Pages/Account/Login.cshtml", model);
    }
}