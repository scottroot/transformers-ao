package.path = package.path .. ";C:/Users/mini/Git/transformers-ao/src/aos/?.lua;C:/Users/mini/Git/transformers-ao/src/aos/.?.lua"
print("before imports")
local loader = require("loader")
local stringify = require(".stringify")
local dump = require(".dump")

local msg = [[{
	"Target": "AOS",
	"Owner": "FOOBAR",
	"Block-Height": "1000",
	"Id": "1234xyxfoo",
	"Module": "WOOPAWOOPA",
	"Tags": [{"name": "Action", "value": "Eval"}],
	"Data": ""
}]]
local env = [[{
    "Process": {
        "Id": "AOS",
        "Owner": "FOOBAR",
        "Tags": [{"name": "Name", "value": "Thomas"}]
    }
}]]
-- instance = loader(msg, env)
-- print(instance)
print(dump(msg))




print("\n###########################\n\t\tFINISHED\n###########################")