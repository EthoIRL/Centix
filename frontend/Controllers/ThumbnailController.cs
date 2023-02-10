using frontend.Api;
using frontend.Api.Models.Media;
using frontend.Models;
using HeyRed.ImageSharp.AVCodecFormats;
using Microsoft.AspNetCore.Mvc;
using SixLabors.ImageSharp;
using SixLabors.ImageSharp.Formats;
using SixLabors.ImageSharp.Formats.Webp;
using SixLabors.ImageSharp.PixelFormats;
using SixLabors.ImageSharp.Processing;

namespace frontend.Controllers;

public class ThumbnailController : Controller
{
    [HttpGet("/thumbnail/")]
    public async Task<ActionResult> Thumbnail([FromQuery] ThumbnailModel model)
    {
        if (ThumbnailExists(model.id))
        {
            var thumbnail = await GetThumbnail(model.id);
            if (thumbnail != null)
            {
                var file = $"{model.id}.{WebpFormat.Instance.FileExtensions.First()}"; 
                return File(thumbnail, WebpFormat.Instance.DefaultMimeType, file);
            }
        }
        else
        {
            ModelContentInfo? contentInfo = await Program.ApiUtils.GetAndReceiveModel<ModelContentInfo>(Program.ConfigManager.Config.BackendApiUri + String.Concat("/media/info?id=", model.id));
            if (contentInfo != null)
            {
                var content = await Program.ApiUtils.GetAndReceiveByteArray(Program.ConfigManager.Config.BackendApiUri + String.Concat("/media/download?id=", model.id));
                if (content != null)
                {
                    var thumbnail = await GetThumbnail(content, 1920, WebpFormat.Instance, contentInfo.content_type);
                
                    SaveThumbnail(model.id, thumbnail);
                    
                    var file = $"{model.id}.{WebpFormat.Instance.FileExtensions.First()}"; 
                    return File(thumbnail, WebpFormat.Instance.DefaultMimeType, file);
                }
            }
        }
        
        return Ok();
    }

    public async Task<byte[]?> GetThumbnail(string id)
    {
        var directory = Path.Join(Environment.CurrentDirectory, "cache");
        var filePath = Path.Join(directory, id);

        if (Directory.Exists(directory))
        {
            if (System.IO.File.Exists(filePath))
            {
                return await System.IO.File.ReadAllBytesAsync(filePath);
            }
        }

        return null;
    }

    public bool ThumbnailExists(string id)
    {
        var directory = Path.Join(Environment.CurrentDirectory, "cache");
        var filePath = Path.Join(directory, id);
        
        if (Path.GetFullPath(filePath) != filePath)
            return false;
        
        if (Directory.Exists(directory))
        {
            if (System.IO.File.Exists(filePath))
            {
                return true;
            }
        }
    
        return false;
    }

    public void SaveThumbnail(string id, byte[] thumbnail)
    {
        var directory = Path.Join(Environment.CurrentDirectory, "cache");
        var filePath = Path.Join(directory, id);
        
        if (Path.GetFullPath(filePath) != filePath)
            return;
        
        Directory.CreateDirectory(Path.GetDirectoryName(filePath)!);
        if (!System.IO.File.Exists(filePath))
        {
            System.IO.File.WriteAllBytes(filePath, thumbnail);
        }
    }

    private static async Task<byte[]> GetThumbnail(byte[] data, int width, IImageFormat format, ModelContentInfo.ContentType contentType)
    {
        Image<Rgba32> image;
        switch (contentType)
        {
            case ModelContentInfo.ContentType.Video:
                var configuration = new Configuration().WithAVDecoders();
                image = Image.Load<Rgba32>(configuration, data);
                break;
            default:
                image = Image.Load<Rgba32>(data);
                break;
        }

        image.Mutate(x => x.Resize(width, image.Height / image.Width * width));

        await using var ms = new MemoryStream();
        await image.SaveAsync(ms, format);
        return ms.ToArray();
    }
}