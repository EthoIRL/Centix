namespace frontend.Api.Models.Media;

public class ModelMediaSearch
{
    public string? username { get; set; }
    public ContentType? content_type { get; set; }
    public string? api_key { get; set; }
    public string[]? tags { get; set; }
    public bool? downloads { get; set; }

    public enum ContentType
    {
        Video,
        Image,
        Other
    }
}