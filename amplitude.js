export const eventProcessor = {
  page(edgee, credentials) {
    /** @type {Map<string, string} */
    let cred = new Map(credentials);
    if (!cred.has("amplitude_api_key")) throw new Error("Missing API KEY");

    let data = { options: {}, events: [] };
    data.api_key = credentials.get("amplitude_api_key");
    data.options.min_id_length = 1;

    if (
      edgee.session_start &&
      edgee.session.previous_session_id !== "" &&
      edgee.session.session_id !== edgee.session.previous_session_id
    ) {
      let event = {
        event_type: "session_end",
        library: "Edgee",
        platform: "Web",
        time: Date.now(),
        session_id: parseInt(edgee.session.previous_session_id, 10) * 1000,
        event_properties: [],
      };

      if (edgee.identify.user_id !== "") {
        event.user_id = edgee.identify.user_id;
      }
      if (edgee.identify.anonymous_id !== "") {
        event.device_id = edgee.identify.anonymous_id;
      } else {
        event.device_id = edgee.identify.edgee_id;
      }
      data.events.push(event);
    }

    let sessionID = parseInt(edgee.session.session_id, 10) * 1000;

    if (edgee.session.session_start) {
      let event = {
        event_type: "session_start",
        library: "Edgee",
        platform: "Web",
        time: Date.now(),
        session_id: sessionID,
        event_properties: [],
      };

      if (edgee.identify.user_id !== "") {
        event.user_id = edgee.identify.user_id;
      }
      if (edgee.identify.anonymous_id !== "") {
        event.device_id = edgee.identify.anonymous_id;
      } else {
        event.device_id = edgee.identify.edgee_id;
      }
      data.events.push(event);
    }

    let event = {
      event_type: "[Amplitude] Page Viewed",
      library: "Edgee",
      platform: "Web",
      time: Date.now(),
      event_properties: [],
      user_agent: edgee.client.user_agent,
      language: edgee.client.locale,
      ip: edgee.client.ip,
      insert_id: edgee.uuid,
      os_name: edgee.client.os_name,
      os_version: edgee.client.os_version,
      device_model: edgee.client.user_agent_model,
    };

    if (sessionID > 0) {
      event.session_id = sessionID;
    }

    if (edgee.identify.user_id !== "") {
      event.user_id = edgee.identify.user_id;
    }
    if (edgee.identify.anonymous_id !== "") {
      event.device_id = edgee.identify.anonymous_id;
    } else {
      event.device_id = edgee.identify.edgee_id;
    }

    if (edgee.page && edgee.page.referrer !== "") {
      let referrerHost = new URL(edgee.page.referrer).host;
      event.event_properties.push(["referrer", edgee.page.referrer]);
      event.event_properties.push(["initial_referrer", edgee.page.referrer]);
      event.event_properties.push(["referring_domain", referrerHost]);
      event.event_properties.push(["initial_referring_domain", referrerHost]);
    }

    data.events.push(event);

    return {
      method: "POST",
      url: "https://api2.amplitude.com/2/httpapi",
      headers: [
        ["content-type", "application/json"],
        ["user-agent", edgee.client.user_agent],
        ["x-forwarded-for", edgee.client.ip],
      ],
      data,
    };
  },

  identify(payload, credentials) {
    return {};
  },

  track(payload, credentials) {
    return {};
  },
};
