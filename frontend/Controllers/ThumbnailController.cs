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
                    bool blur = false;
                    
                    if (Program.ConfigManager.Config.BlurTags.Length > 0 && contentInfo.tags != null)
                    {
                        foreach (var contentInfoTag in contentInfo.tags)
                        {
                            if (Program.ConfigManager.Config.BlurTags.ToList().ConvertAll(tag => tag.ToLower())
                                .Any(tag => tag == contentInfoTag.ToLower()))
                            {
                                blur = true;
                            }
                        }
                    }
                    
                    var thumbnailImage = GenerateThumbnail(content, 450, contentInfo.content_type);
                    
                    if (blur)
                    {
                        thumbnailImage = BlurImage(thumbnailImage);
                    }

                    var thumbnail = await SaveImage(thumbnailImage, WebpFormat.Instance);
                    
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

    private static Image<Rgba32> LoadImage(byte[] data, ModelContentInfo.ContentType contentType)
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
        
        return image;
    }

    private static Image<Rgba32> GenerateThumbnail(byte[] data, int width, ModelContentInfo.ContentType contentType)
    {
        Image<Rgba32> image = LoadImage(data, contentType);

        var padColor = Color.Transparent;
        if (!Program.ConfigManager.Config.TransparentThumbnailPadding)
        {
            padColor = Color.Black;
        }

        var resizeOptions = new ResizeOptions
        {
            Mode = ResizeMode.Pad,
            PadColor = padColor,
            Size = new Size(width, 250)
        };
        image.Mutate(x => x.Resize(resizeOptions));

        return image;
    }

    private static async Task<byte[]> SaveImage(Image<Rgba32> image, IImageFormat format)
    {
        await using var ms = new MemoryStream();
        await image.SaveAsync(ms, format);
        return ms.ToArray();
    }

    private static Image<Rgba32> BlurImage(Image<Rgba32> image, int sigma = 16)
    {
        image.Mutate(x => x.GaussianBlur(sigma));
        
        return image;
    }
}