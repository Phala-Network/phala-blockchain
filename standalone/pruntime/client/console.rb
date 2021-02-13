#!/usr/bin/env ruby
# frozen_string_literal: true

ENV["BUNDLE_GEMFILE"] ||= File.expand_path("Gemfile", __dir__)

require "bundler/setup" # Set up gems listed in the Gemfile.

require 'active_support'
require 'active_support/core_ext'

require "pry"

require "faraday"
require "faraday_middleware"

HOST = "http://localhost:8000/"

class APIClient
  class APIError < StandardError; end

  attr_reader :client

  def initialize(host:)
    @client = Faraday.new(HOST) do |faraday|
      faraday.request :json
      faraday.options[:open_timeout] = 2
      faraday.options[:timeout] = 5
      
      # faraday.response :logger
      # faraday.response :json
      
      faraday.adapter Faraday.default_adapter
    end
  end

  def get_info
    response = client.post("get_info", {input: {}, nonce: {}})

    if response.status == 500
      raise APIError, "Unexpected return code 500"
    end

    body =
      begin
        JSON.parse(response.body)
      rescue
        raise APIError, "Parse body error -- #{response.body}"
      end

    if body["payload"].present?
      begin
        body["payload"] = JSON.parse(body["payload"]) 
      rescue
        raise APIError, "Parse payload error -- #{body["payload"]}"
      end
    end

    body
  end
end

client = APIClient.new host: HOST

# start a REPL session
binding.pry
