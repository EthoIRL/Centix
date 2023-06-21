﻿using System.Diagnostics;
using Microsoft.AspNetCore.Http.Extensions;

namespace frontend.Analytics;

public class AnalyticsMiddleware
{
    private readonly RequestDelegate _next;

    private readonly AnalyticsApi _analyticsApi;

    public AnalyticsMiddleware(RequestDelegate next, string apiKey)
    {
        _analyticsApi = new AnalyticsApi(apiKey);
        
        _next = next;
    }
    
    public async Task InvokeAsync(HttpContext context)
    {
        string hostname = context.Request.GetDisplayUrl();
        string ipAddress = context.Connection.RemoteIpAddress?.ToString() ?? String.Empty;
        string method = context.Request.Method;
        string userAgent = context.Request.Headers.UserAgent;
        string path = context.Request.Path;

        int statusCode = context.Response.StatusCode;
        var createdAt = DateTime.UtcNow.ToString("yyyy-MM-dd'T'HH:mm:ss.fffK");
        
        var watch = new Stopwatch(); 
            
        watch.Start();  
        context.Response.OnStarting(() =>
        {
            watch.Stop();
            var responseTime = (int) watch.ElapsedMilliseconds;

            Analytics analytics = new Analytics
            {
                hostname = hostname,
                ip_address = ipAddress,
                path = path,
                user_agent = userAgent,
                method = method,
                response_time = responseTime,
                status = statusCode,
                created_at = createdAt
            };
            
            _analyticsApi.LogRequest(analytics);

            return Task.CompletedTask;  
        });
        
        // Call the next delegate/middleware in the pipeline.
        await _next(context);
    }
    
}

public static class AnalyticsMiddlewareExtensions
{
    public static void UseAnalyticsMiddleware(this IApplicationBuilder builder, string apiKey)
    {
        builder.UseMiddleware<AnalyticsMiddleware>(apiKey);
    }
}