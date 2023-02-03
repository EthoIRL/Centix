namespace frontend.Api.Models.User;

public class ModelUserInfo
{
    public string username { get; set; }
    public string creation_date { get; set; }
    public string[] uploads { get; set; }
    public int total_upload_size { get; set; }
    public bool admin { get; set; }
    public string? invite_key { get; set; }
}