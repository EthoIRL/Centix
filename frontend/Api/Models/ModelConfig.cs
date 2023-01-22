using System.Text;

namespace frontend.Api.Models;

public class ModelConfig
{
    public bool allow_content_editing { get; set; }
    public bool allow_custom_tags { get; set; }
    public bool allow_user_registration { get; set; }
    public int content_id_length { get; set; }
    public int content_max_size { get; set; }
    public int content_name_length { get; set; }
    public int custom_tag_length { get; set; }
    public string[] domains { get; set; }
    public bool use_invite_keys { get; set; }
    public int user_upload_limit { get; set; }
}