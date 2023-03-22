using frontend.Api.Models.Media;
using frontend.Models;
using Microsoft.AspNetCore.Mvc;

namespace frontend.Controllers;

public class DownloadController : Controller
{
    // TODO: Implement caching
    [HttpGet("/download/")]
    public async Task<ActionResult> Download([FromQuery] DownloadModel model)
    {
        ModelContentInfo? contentInfo = await Program.ApiUtils.GetAndReceiveModel<ModelContentInfo>(Program.ConfigManager.Config.BackendApiUri + String.Concat("/media/info?id=", model.id));
        if (contentInfo != null)
        {
            var content = await Program.ApiUtils.GetAndReceiveByteArray(Program.ConfigManager.Config.BackendApiUri + String.Concat("/media/download?id=", model.id));
            if (content != null)
            {
                var file = $"{contentInfo.content_name}.{contentInfo.content_extension}";
                return File(content, "application/octet-stream", file);
            }
        }

        return BadRequest();
    }
}