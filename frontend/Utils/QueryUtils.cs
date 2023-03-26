namespace frontend.Utils;

public static class QueryUtils
{
    public static bool IsSelectedFromSortQuery(IQueryCollection queryCollection, String sortValue)
    {
        if (queryCollection.Count > 0)
        {
            foreach (var collection in queryCollection)
            {
                if (collection.Key.ToLower() == sortValue.ToLower())
                {
                    if (collection.Value[0] == null)
                        continue;
                    Boolean.TryParse(collection.Value[0], out bool value);
                    return value;
                }
            }
        }

        return false;
    }
}