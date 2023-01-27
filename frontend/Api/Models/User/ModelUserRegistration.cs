namespace frontend.Api.Models.User;

public class ModelUserRegistration
{
    public string? invite { get; set; }
    
    public UserCredentials user_credentials { get; set; }
    public class UserCredentials
    {
        public string username { get; set; }
        public string password { get; set; }
    }
}