local json = require "json"
local process = require ".process"
ao = require "ao"

function handle(msgJSON, aoJSON)
    -- decode inputs
    local msg = json.decode(msgJSON)
    local env = json.decode(aoJSON)
    ao.init(env)
    -- relocate custom tags to root message
    msg = ao.normalize(msg)
    local status, response = pcall(function()
        return (process.handle(msg, ao))
    end)

    -- encode output
    local responseJSON = json.encode({ok = status, response = response})
    return responseJSON
end