namespace frontend.Models.Account;

public class SignupModel
{
    public string username { get; set; }
    public string password { get; set; }
    public string? invite { get; set; }
}