const make_request = (method, url, options) => {
    const init = {
        method,
        headers: {}
    }

    if (options.headers) Object.assign(init.headers, options.headers)

    if (method === 'POST' || method === 'PATCH' || method === 'PUT')
        init.body =
            init.headers['Content-Type'] === 'application/json'
                ? JSON.stringify(options.body)
                : options.body

    // if (options.base_url !== undefined) base_url = options.base_url

    return fetch(url, init)
}

export const get = (url, options = {}) =>
    new Promise((resolve, reject) => {
        make_request('GET', url, options).then(response => {
            if (!options.validate_status)
                options.validate_status = status => status === 200

            if (options.validate_status && !options.validate_status(response.status))
                reject(response)

            resolve(response)
        })
        .catch(e => reject(e))
    })

export const post = (url, options = {}) =>
    new Promise((resolve, reject) => {
        make_request('POST', url, options).then(response => {
            if (!options.validate_status)
                options.validate_status = status => status === 201

            if (options.validate_status && !options.validate_status(response.status))
                reject(response)

            resolve(response)
        })
        .catch(e => reject(e))
    })

export const patch = (url, options = {}) =>
    new Promise((resolve, reject) => {
        make_request('PATCH', url, options).then(response => {
            if (!options.validate_status)
                options.validate_status = status => status === 200

            if (options.validate_status && !options.validate_status(response.status))
                reject(response)

            resolve(response)
        })
        .catch(e => reject(e))
    })

export const put = (url, options = {}) =>
    new Promise((resolve, reject) => {
        make_request('PUT', url, options).then(response => {
            if (!options.validate_status)
                options.validate_status = status => status === 200

            if (options.validate_status && !options.validate_status(response.status))
                reject(response)

            resolve(response)
        })
        .catch(e => reject(e))
    })

export const del = (url, options = {}) =>
    new Promise((resolve, reject) => {
        make_request('DELETE', url, options).then(response => {
            if (!options.validate_status)
                options.validate_status = status => status === 200

            if (options.validate_status && !options.validate_status(response.status))
                reject(response)

            resolve(response)
        })
        .catch(e => reject(e))
    })
  