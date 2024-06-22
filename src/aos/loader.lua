-- package.path = package.path .. ";C:/Users/mini/Git/transformers-ao/src/aos/?.lua;C:/Users/mini/Git/transformers-ao/src/aos/.?.lua"

local json = require("json")
ao = require("ao")
local process = require(".process")


local function handle(msgJSON, aoJSON)
    -- print(msgJSON)
    -- decode inputs
    local msg = json.decode(msgJSON)
    local env = json.decode(aoJSON)
    ao.init(env)
    -- -- relocate custom tags to root message
    msg = ao.normalize(msg)
    local status, response = pcall(function()
        return (process.handle(msg, ao))
    end)

    -- encode output
    local responseJSON = json.encode({ok = status, response = response})
    -- local responseJSON = "blah blah blah"
    return responseJSON
end
return handle


-- local env = [[{
--     "Process": {
--         "Id": "AOS",
--         "Owner": "FOOBAR",
--         "Tags": [{"name": "Name", "value": "Thomas"}]
--     }
-- }]]

-- local pingpong = [[
--     Handlers.add('pingpong', 
--         Handlers.utils.hasMatchingData('ping'), 
--         function(Msg) 
--             print('pong') 
--         end
--     )
-- ]]
-- local msg1 = [[{
--     "Target": "AOS",
--     "Owner": "FOOBAR",
--     "Block-Height": "1000",
--     "Id": "1234xyxfoo",
--     "Module": "WOOPAWOOPA",
--     "Tags": [{"name": "Action", "value": "Eval"}],
--     "Data": "Handlers.append(\"pingpong\", Handlers.utils.hasMatchingTag(\"Action\", \"ping\"), function(Msg) return 'pong' end)"
-- }]]

-- local result1 = handle(msg1, env)
-- println("Result 1 = " .. (json.encode(json.decode(result1)["response"]["Output"]["data"]) or "no data"))

-- -- print(Handlers)
-- local msg2 = [[{
--     "Target": "AOS",
--     "Owner": "FOOBAR",
--     "Block-Height": "1000",
--     "Id": "1234xyxfoo",
--     "Module": "WOOPAWOOPA",
--     "Tags": [{"name": "Action", "value": "ping"}],
--     "Data": ""
-- }]]
-- local result2 = handle(msg2, env)
-- -- print("Result 2 = " .. json.decode(msg2))
-- println("Result 2 = " .. (json.decode(result2)["response"]["Output"] or "no data"))
-- println(result2)