namespace frontend.Api.Models.Media;

public class ModelContentInfo
{
    public string author_username { get; set; }
    public string content_name { get; set; }
    public int content_size { get; set; }
    public string upload_date { get; set; }
    public bool unlisted { get; set; }
    public string[]? tags { get; set; }
    public int downloads { get; set; }
}