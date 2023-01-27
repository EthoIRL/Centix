namespace frontend.Api.Models;

public class ModelConfig
{
    public bool media_allow_editing { get; set; }
    public int media_max_name_length { get; set; }
    public string[] backend_domains { get; set; }
    public string[] tags_default { get; set; }
    public bool tags_allow_custom { get; set; }
    public int tags_max_name_length { get; set; }
    public bool registration_allow { get; set; }
    public bool registration_use_invite_keys { get; set; }
    public int user_upload_limit { get; set; }
    public int user_upload_size_limit { get; set; }
    public int user_total_upload_size_limit { get; set; }
    public int user_username_limit { get; set; }
    public int user_password_limit { get; set; }
}