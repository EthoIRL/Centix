namespace frontend.Config;

public class Config
{
    /// <summary>
    /// URL of backend api
    /// Example: http://127.0.0.1:8000/api/
    /// <remarks>
    /// Preferably don't use a domain as the initial/first requests can take up to multiple seconds even on localhost due to DNS lookup
    /// https://stackoverflow.com/questions/62352026/net-core-3-1-ihttpclientfactory-httpclient-slow-on-first-request
    /// </remarks>
    /// </summary>
    public string BackendApiUri { get; set; } = String.Empty;
}