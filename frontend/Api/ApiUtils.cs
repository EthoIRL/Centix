namespace frontend.Api;

public static class ApiUtils
{
    static readonly HttpClient Client = new();
    
    public static async Task<T?> GetModelAsync<T>(string path)
    {
        T? model = default(T);
        
        HttpResponseMessage response = await Client.GetAsync(path);
        if (response.IsSuccessStatusCode)
        {
            model = await response.Content.ReadAsAsync<T>();
        }
        
        return model;
    }
}