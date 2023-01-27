﻿using System.Net;
using System.Net.Http.Headers;
using frontend.Utils;

namespace frontend.Api;

public class ApiUtils
{
    private readonly HttpClient _client;

    public ApiUtils()
    {
        var socketsHandler = new SocketsHttpHandler
        {
            PooledConnectionLifetime = Timeout.InfiniteTimeSpan,
            PooledConnectionIdleTimeout = Timeout.InfiniteTimeSpan,
        };

        _client = new HttpClient(socketsHandler);
        _client.DefaultRequestHeaders.CacheControl = new CacheControlHeaderValue()
        {
            NoCache = true
        };
    }

    /// <summary>
    /// Grabs information and serializes it into a api model 
    /// </summary>
    /// <param name="path">Path leading to backend api URL method</param>
    /// <typeparam name="T">The type of model to attempt to serialize too</typeparam>
    /// <returns></returns>

    public async Task<T?> GetModelAsync<T>(string path)
    {
        T? model = default(T);

        Console.WriteLine($"Requesting model URL: \"{path}\"");

        try
        {
            HttpResponseMessage response = await _client.GetAsync(path);
            if (response.IsSuccessStatusCode)
            {
                model = await response.Content.ReadAsAsync<T>();
            }
        }
        catch (Exception exception)
        {
            if (exception is not HttpRequestException)
            {
                Console.WriteLine(exception);
            }
        }

        return model;
    }
    
    /// <summary>
    /// Sends information to a given URL and attempts to receive a model
    /// </summary>
    /// <param name="path">Path leading to backend api URL method</param>
    /// <param name="model">Any API Model</param>
    /// <typeparam name="T">Post model (e.g. ModelLogin) </typeparam>
    /// <typeparam name="TB">Body Receiving (e.g. ModelLoginResponse) </typeparam>
    /// <returns></returns>
    public async Task<TB?> PostAndReceiveModel<T, TB>(string path, T model)
    {
        TB? returnModel = default(TB);
        
        Console.WriteLine($"Requesting model URL: \"{path}\"");
        
        try
        {
            HttpResponseMessage response = await _client.PostAsJsonAsync(path, model);
            if (response.IsSuccessStatusCode)
            {
                returnModel = await response.Content.ReadFromJsonAsync<TB>();
            }
        } 
        catch (Exception exception)
        {
            if (exception is not HttpRequestException)
            {
                Console.WriteLine(exception);
            }
        }
        
        return returnModel;
    }
}