local stringify = require(".stringify")
local json = require("json")


-- handler for eval
local function eval(aoInstance)
  if not aoInstance.outbox then
    aoInstance.outbox = {}
  end
  if not Prompt then
    Prompt = function ()
      return "aos> "
    end
  end
  return function (msg)
    -- exec expression
    local expr = msg.Data
    local func, err = load("return " .. expr, 'aos', 't', _G)
    local output = ""
    local e = nil

    if err then
      func, err = load(expr, 'aos', 't', _G)
    end
    if func then
      -- output, e = func()
      local success
      success, output = pcall(func)

      if not success then
        aoInstance.outbox.Error = output
        return
      end
    else
      aoInstance.outbox.Error = err
      return
    end
    if e then 
      aoInstance.outbox.Error = e
      return 
    end
    -- set result in outbox.Output
    local json_string = "undefined"
    local output_string = output
    if type(output) == "table" then
    	output_string = stringify.format(output)
    	local jstatus, jresult = pcall(function () return json.encode(output) end)
    	if jstatus then
    		json_string = jresult
    	end
    end
    aoInstance.outbox.Output = {
      data = {
        --json = type(output) == "table" and pcall(function () return json.encode(output) end) and output or "undefined",
        json = json_string,
        --output = (type(output) == "table" and stringify.format(output)) or output,
        output = output_string,
        prompt = Prompt()
      }
    }
  end
end

return eval
-- x = {
--   id = "ABC",
--   _version = "0.0.4",
--   _module = "ABC",
--   outboxtable = {},
-- }

-- ao = require("ao")
-- y = _eval(ao)

-- print(y({Data = "print('yes')", outbox = {}}))