local pretty = require('.pretty')
local base64 = require('.base64')
local json = require('json')
local chance = require('.chance')
local stringify = require(".stringify")
local _ao = require('ao')
-- local crypto = require('.crypto.init')
Dump = require('.dump')
Utils = require('.utils')
Handlers = require('.handlers')

Colors = {
  red = "\27[31m",
  green = "\27[32m",
  blue = "\27[34m",
  reset = "\27[0m",
  gray = "\27[90m"
}
Bell = "\x07"
Errors = Errors or {}
Inbox = Inbox or {}

local process = { _version = "0.2.0" }

local maxInboxCount = 10000

-- wrap ao.send and ao.spawn for magic table
local aosend = _ao.send 
local aospawn = _ao.spawn
_ao.send = function (msg)
  if msg.Data and type(msg.Data) == 'table' then
    msg['Content-Type'] = 'application/json'
    msg.Data = require('json').encode(msg.Data)
  end
  return aosend(msg)
end

_ao.spawn = function (module, msg) 
  if msg.Data and type(msg.Data) == 'table' then
    msg['Content-Type'] = 'application/json'
    msg.Data = require('json').encode(msg.Data)
  end
  return aospawn(module, msg)
end

local function insertInbox(msg)
  table.insert(Inbox, msg)
  if #Inbox > maxInboxCount then
    local overflow = #Inbox - maxInboxCount 
    for i = 1,overflow do
      table.remove(Inbox, 1)
    end
  end 
end

local function findObject(array, key, value)
  for i, object in ipairs(array) do
    if object[key] == value then
      return object
    end
  end
  return nil
end

function Tab(msg)
  local inputs = {}
  for _, o in ipairs(msg.Tags) do
    if not inputs[o.name] then
      inputs[o.name] = o.value
    end
  end
  return inputs
end

function Prompt()
  return "aos> "
end

if not println then
  println = print
end

function print(a)
  if type(a) == "table" then
    a = stringify.format(a)
  end
  
  pcall(function () 
    local data = a
    if _ao.outbox.Output.data then
      data = _ao.outbox.Output.data .. "\n" .. a
    end
    _ao.outbox.Output = { data = data, prompt = Prompt(), print = true }
  end)
  -- println(a)
  return tostring(a)
end

function Send(msg)
  _ao.send(msg)
  return 'message added to outbox'
end

function Spawn(module, msg)
  if not msg then
    msg = {}
  end

  _ao.spawn(module, msg)
  return 'spawn process request'
end

function Assign(assignment)
  _ao.assign(assignment)
  return 'assignment added to outbox'
end

Seeded = Seeded or false

-- this is a temporary approach...
local function stringToSeed(s)
  local seed = 0
  for i = 1, #s do
      local char = string.byte(s, i)
      seed = seed + char
  end
  return seed
end

local function initializeState(msg, env)
  if not Seeded then
    --math.randomseed(1234)
    local height = msg['Block-Height'] or "1000"
    -- chance.seed(tonumber(msg['Block-Height'] .. stringToSeed(msg.Owner .. msg.Module .. msg.Id)))
    chance.seed(12)
    math.random = function (...)
      local args = {...}
      local n = #args
      if n == 0 then
        return chance.random()
      end
      if n == 1 then
        return chance.integer(1, args[1])
      end
      if n == 2 then
        return chance.integer(args[1], args[2])
      end
      return chance.random()
    end
    Seeded = true
  end
  Errors = Errors or {}
  Inbox = Inbox or {}

  -- temporary fix for Spawn
  if not Owner then
    local _from = findObject(env.Process.Tags, "name", "From-Process")
    if _from then
      Owner = _from.value
    else
      Owner = msg.From
    end
  end
  if not Name then
    local aosName = findObject(env.Process.Tags, "name", "Name")
    if aosName then
      Name = aosName.value
    else
      Name = 'aos'
    end
  end
end

function Version()
  print("version: " .. process._version)
end

function process.handle(msg, ao)
  ao.id = ao.env.Process.Id
  initializeState(msg, ao.env)

  -- tagify msg
  msg.TagArray = msg.Tags
  msg.Tags = Tab(msg)
  -- tagify Process
  ao.env.Process.TagArray = ao.env.Process.Tags
  ao.env.Process.Tags = Tab(ao.env.Process)
  -- if type(msg) == "table" then
  -- 	println(require(".stringify").format(msg))
  -- end

  --println(tostring(msg.Tags))
  -- if msg.Tags['Model-Type'] and msg.Tags['Model-Type'] == 'bert' then
  --   -- println("Tags Model-Type matches 'bert'")
  --   local bert = require("bert")
  --   if bert == nil then
  --     print("bert is not here man")
  --     return
  --   end
  --   -- println("Required 'bert'")
  --   local config = {}
  --   config.prompt = msg.Data
  --   --msg.Data = require('json').decode(msg.Data or "{}")
  --   local embedding = bert.encode_text(config)
  --   -- println("Got embedding from 'bert'")
  --   --local result = json.encode(embedding)
  --   --println("finished result")
  --   return {Output = embedding}
  -- end
  -- magic table - if Content-Type == application/json - decode msg.Data to a Table
  if msg.Tags['Content-Type'] and msg.Tags['Content-Type'] == 'application/json' then
    msg.Data = require('json').decode(msg.Data or "{}")
  end
  if msg.Tags['Action'] and msg.Tags['Action'] == 'ping' then
    return {Output = "pong"}
  end
  -- init Errors
  Errors = Errors or {}
  -- clear Outbox
  ao.clearOutbox()

  Handlers.add("_eval",
    --function (msg)
    function ()
      return msg.Action == "Eval" and Owner == msg.From
    end,
    require('.eval')(ao)
  )
  Handlers.append("_default",
    function () return true end,
    require('.default')(insertInbox)
  )

  local status, result = pcall(Handlers.evaluate, msg, ao.env)

  if not status then
    table.insert(Errors, result)
    return { Error = result }
    -- return {
    --   Output = { data = {prompt = Prompt(), json = 'undefined', output = result} },
    --   Messages = {}, 
    --   Spawns = {}
    -- }
  end

  --FOR REFERENCE -- THE ao.result() FUNCTION IS:
          --function ao.result(result)
          --    if ao.outbox.Error or result.Error then
          --        return {Error = result.Error or ao.outbox.Error}
          --    end
          --    return {
          --        Output = result.Output or ao.outbox.Output,
          --        Messages = ao.outbox.Messages,
          --        Spawns = ao.outbox.Spawns,
          --        Assignments = ao.outbox.Assignments
          --    }
          --end
  return ao.result({ })
end

return process