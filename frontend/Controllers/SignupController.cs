using System.Net;
using frontend.Api.Models;
using frontend.Api.Models.User;
using frontend.Api.Models.User.Login;
using frontend.Models.Account;
using frontend.Utils;
using Microsoft.AspNetCore.Mvc;

namespace frontend.Controllers;

public class SignupController : Controller
{

    [HttpPost("/signup/")]
    public async Task<ActionResult> SignUp([FromForm] SignupModel model)
    {
        if (ModelState.IsValid)
        {
            var modelUserRegistration = new ModelUserRegistration
            {
                invite = model.invite,
                user_credentials = new ModelUserRegistration.UserCredentials
                {
                    username = model.username,
                    password = model.password
                }
            };
            
            var response =
                await Program.ApiUtils.PostAndReceiveResponse(
                    Program.ConfigManager.Config.BackendApiUri + "/user/register", modelUserRegistration);

            if (response?.StatusCode == HttpStatusCode.OK)
            {
                var userLoginResponse =
                    await Program.ApiUtils.PostAndReceiveResponse(
                        Program.ConfigManager.Config.BackendApiUri + "/user/login", modelUserRegistration.user_credentials);

                if (userLoginResponse?.StatusCode == HttpStatusCode.OK)
                {
                    var loginResponse = await userLoginResponse.Content.ReadFromJsonAsync<ModelLoginResponse>();

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
            }

            ModelError? error = null;
            if (response != null)
            {
                await response.Content.ReadFromJsonAsync<ModelError>();
            }

            ModelState.AddModelError("Error", 
                error != null ? error.error : "An internal error has occured within the server!");
        }

        if (ModelState.ErrorCount == 0)
        {
            ModelState.AddModelError("Error", "An internal error has occured within the server!");
        }
        return View("~/Web/Pages/Account/Signup.cshtml", model);
    }
}