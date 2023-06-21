namespace frontend.Config;

public class Config
{
    /// <summary>
    /// URL of backend api
    /// Example: http://127.0.0.1:8000/api
    /// <remarks>
    /// Preferably don't use a domain as the initial/first requests can take up to multiple seconds even on localhost due to DNS lookup
    /// https://stackoverflow.com/questions/62352026/net-core-3-1-ihttpclientfactory-httpclient-slow-on-first-request
    /// </remarks>
    /// </summary>
    public string BackendApiUri { get; set; } = String.Empty;
    
    /// <summary>
    /// Use transparency for padding or black bars for thumbnail images
    /// </summary>
    public bool TransparentThumbnailPadding { get; set; } = true;

    /// <summary>
    /// What thumbnails to blur based on tags
    /// </summary>
    public string[] BlurTags { get; set; } = {"nsfw"};
    
    /// <summary>
    /// Text/HTML Snippet inserted into the announcement container
    /// </summary>
    /// <remarks>
    /// Html A tag is supported
    /// </remarks>
    public string[] Announcements { get; set; } =
    {
        "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum. - Mar 30 2023",
        @"This is an invite only service! (<a href=""https://google.com/"">https://google.com/</a>) - Mar 30 2023"
    };

    public string? AnalyticsApi { get; set; } = null;
}