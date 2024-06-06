package.path = package.path .. ";./?.lua"
stringify = require("stringify")
PRINT = print
function println(x)
  if type(x) == "table" then
  	PRINT(stringify.format(x))
    return
  end
  PRINT(x)
end
local json = require "json"
local process = require ".process"
ao = require "ao"

function handle(msgJSON, aoJSON)
    -- decode inputs
    local msg = json.decode(msgJSON)
    print(require("stringify").format(msg))
    local env = json.decode(aoJSON)
    ao.init(env)
    -- relocate custom tags to root message
    msg = ao.normalize(msg)
    local status, response = pcall(function()
        local r = process.handle(msg, ao)
        println(r)
        return (r)
    end)
    local responseJSON = json.encode({ok = status, response = response})
    return responseJSON
end

local _msg = '{ "Owner": "tom", "Block-Height": 1000, "Id": "1234xyxfoo", "Module": "WOOPAWOOPA", "Target": "1", "Tags": [{ "name": "Action", "value": "inc" }], "Data": "" }'
local _env = '{ "Process": { "Id": "Ab18dE", "Owner": "FOOBAR", "Tags": [{ "name": "Name", "value": "Thomas" }] } }'
local r = handle(_msg, _env)
println(stringify.format(json.decode(r), 2))
