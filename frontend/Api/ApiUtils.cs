namespace frontend.Api;

public class ApiUtils
{
    private readonly HttpClient _client;

    public ApiUtils()
    {
        var socketsHandler = new SocketsHttpHandler
        {
            PooledConnectionLifetime = Timeout.InfiniteTimeSpan,
            PooledConnectionIdleTimeout = Timeout.InfiniteTimeSpan
        };

        _client = new HttpClient(socketsHandler);
    }

    
    public async Task<T?> GetModelAsync<T>(string path)
    {
        T? model = default(T);
        
        Console.WriteLine($"Requesting model URL: \"{path}\"");
        
        HttpResponseMessage response = await _client.GetAsync(path);
        if (response.IsSuccessStatusCode)
        {
            model = await response.Content.ReadAsAsync<T>();
        }
        
        return model;
    }
}