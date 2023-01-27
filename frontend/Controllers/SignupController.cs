using System.Net;
using frontend.Api.Models.User;
using frontend.Models.Account;
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
                return Redirect("~/account");
            }

            if (response != null)
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
        return View("~/Web/Pages/Account/Signup.cshtml", model);
    }
}