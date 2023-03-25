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
    // TODO: Implement browser image caching
    // TODO: Thumbnails could stack up infinitely causing issues since they aren't being deleted
    // (10kb~ Average per image so not really an issue)
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
            if (contentInfo != null && contentInfo.content_type != ModelContentInfo.ContentType.Other)
            {
                var content = await Program.ApiUtils.GetAndReceiveByteArray(Program.ConfigManager.Config.BackendApiUri + String.Concat("/media/download?id=", model.id));
                if (content != null)
                {
                    // TODO: Optional config feature
                    bool? blur = contentInfo.tags?.Contains("nsfw");
                    var thumbnail = await GetThumbnail(content, 480, WebpFormat.Instance, contentInfo.content_type, blur ?? false);

                    SaveThumbnail(model.id, thumbnail);
                    
                    var file = $"{model.id}.{WebpFormat.Instance.FileExtensions.First()}"; 
                    return File(thumbnail, WebpFormat.Instance.DefaultMimeType, file);
                }
            }
        }
        
        var placeholderThumbnail = await GetPlaceholderThumbnail();
        if (placeholderThumbnail != null)
        {
            var placeholderFile = $"{model.id}.{WebpFormat.Instance.FileExtensions.First()}"; 
            return File(placeholderThumbnail, WebpFormat.Instance.DefaultMimeType, placeholderFile);
        }
        return BadRequest();
    }

    public async Task<byte[]?> GetThumbnail(string id)
    {
        var directory = Path.Join(Environment.CurrentDirectory, "cache");
        var filePath = Path.Join(directory, id);
        filePath = Path.ChangeExtension(filePath, WebpFormat.Instance.FileExtensions.First());

        if (Directory.Exists(directory))
        {
            if (System.IO.File.Exists(filePath))
            {
                return await System.IO.File.ReadAllBytesAsync(filePath);
            }
        }

        return null;
    }
    
    public async Task<byte[]?> GetPlaceholderThumbnail()
    {
        var directory = Path.Join(Environment.CurrentDirectory, "Web", "Assets", "imgs");
        var filePath = Path.Join(directory, "placeholder.webp");

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
        filePath = Path.ChangeExtension(filePath, WebpFormat.Instance.FileExtensions.First());
        
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
        filePath = Path.ChangeExtension(filePath, WebpFormat.Instance.FileExtensions.First());
        
        if (Path.GetFullPath(filePath) != filePath)
            return;
        
        Directory.CreateDirectory(Path.GetDirectoryName(filePath)!);
        if (!System.IO.File.Exists(filePath))
        {
            System.IO.File.WriteAllBytes(filePath, thumbnail);
        }
    }

    private static async Task<byte[]> GetThumbnail(byte[] data, int width, IImageFormat format, ModelContentInfo.ContentType contentType, bool blur = false)
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

        var resizeOptions = new ResizeOptions
        {
            Mode = ResizeMode.Pad,
            PadColor = Color.Black,
            Size = new Size(width, image.Height / image.Width * width)
        };
        image.Mutate(x => x.Resize(resizeOptions));
        if (blur)
        {
            image.Mutate(x => x.GaussianBlur(12));
        }
        
        await using var ms = new MemoryStream();
        await image.SaveAsync(ms, format);
        return ms.ToArray();
    }
}