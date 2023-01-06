
pi = 3.1415926

def orifice_incompressible(do, dp, Cd, p1, p2, rho):
    """Orifice flow in incompressible fluid.

    Parameters
    ----------
    do : float
        Orifice diameter.
    dp : float
        Pipe diameter.
    p1 : float
        Upstream pressure.
    p2 : float
        Downstream pressure.
    rho : float
        Fluid density.

    Returns
    -------
    m : float
        Mass flow rate.
    """

    beta = do / dp
    Cf = Cd / pow(1.0 - pow(beta, 4.0), 0.5)
    orifice_area = pi * pow(do / 2.0, 2.0)

    return Cf * orifice_area * pow(2.0 * rho * (p1 - p2), 0.5)

def orifice_incompressible_diameter(m, dp, Cd, p1, p2, rho):
    """Orifice flow in incompressible fluid.

    Parameters
    ----------
    m : float
        Mass flow rate.
    dp : float
        Pipe diameter.
    p1 : float
        Upstream pressure.
    p2 : float
        Downstream pressure.
    rho : float
        Fluid density.

    Returns
    -------
    do : float
        Orifice diameter
    """

    upper = (2**(3/4)) * dp * (m**0.5)
    lower = pow(pi**2 * Cd**2 * dp**4 * (p1 - p2) * rho + 8 * m**2, 0.25)

    return upper / lower

def orifice_incompressible_pressure(m, do, dp, Cd, p2, rho):
    """Orifice flow in incompressible fluid.

    Parameters
    ----------
    m : float
        Mass flow rate.
    do : float
        Orifice diameter.
    dp : float
        Pipe diameter.
    p2 : float
        Downstream pressure.
    rho : float
        Fluid density.

    Returns
    -------
    p1 : float
        Upstream pressure.
    """

    beta = do / dp
    Cf = Cd / pow(1.0 - pow(beta, 4.0), 0.5)
    orifice_area = pi * pow(do / 2.0, 2.0)

    return pow(m / (Cf * orifice_area), 2.0) / (2.0 * rho) + p2

def orifice_compressible(do, dp, Cd, p1, p2, rho, kappa=1.3):
    """Orifice flow in compressible fluid.

    Parameters
    ----------
    do : float
        Orifice diameter.
    dp : float
        Pipe diameter.
    p1 : float
        Upstream pressure.
    p2 : float
        Downstream pressure.
    rho : float
        Upstream density.
    kappa : float
        Isentropic exponent, can be approximated by specific heat ratio.

    Returns
    -------
    m : float
        Mass flow rate.
    """

    beta = do / dp
    if beta <= 0.25:
        pass # print("Flow is probably choked, results aren't accurate! (beta is {})".format(beta))

    Cf = Cd / pow(1.0 - pow(beta, 4.0), 0.5)
    orifice_area = 3.1415926 * pow(do / 2.0, 2.0)
    e = 1.0 - (0.351 + 0.256 * pow(beta, 4.0) + 0.93 * pow(beta, 8.0)) * (1.0 - pow(p2 / p1, 1.0 / kappa))

    return Cf * e * orifice_area * pow(2.0 * rho * (p1 - p2), 0.5)

def orifice_compressible_diameter(m, dp, Cd, p1, p2, rho, kappa=1.3):
    """Orifice flow in compressible fluid.

    Parameters
    ----------
    m : float
        Mass flow rate.
    dp : float
        Pipe diameter.
    p1 : float
        Upstream pressure.
    p2 : float
        Downstream pressure.
    rho : float
        Upstream density.
    kappa : float
        Isentropic exponent, can be approximated by specific heat ratio.

    Returns
    -------
    do : float
        Orifice diameter.
    """

    approx_diameter = orifice_incompressible_diameter(m, dp, Cd, p1, p2, rho)
    approx_m = orifice_compressible(approx_diameter, dp, Cd, p1, p2, rho, kappa)
    epsilon = abs(m - approx_m) * 1e-5

    while abs(approx_m - m) > epsilon:
        delta = ((m - approx_m) / m) * approx_diameter * 0.1
        approx_diameter += delta

        approx_m = orifice_compressible(approx_diameter, dp, Cd, p1, p2, rho, kappa)

    return approx_diameter

def orifice_compressible_pressure(m, do, dp, Cd, p2, rho, kappa=1.3, debug_print=False):
    """Orifice flow in compressible fluid.

    Parameters
    ----------
    m : float
        Mass flow rate.
    do : float
        Orifice diameter.
    dp : float
        Pipe diameter.
    p2 : float
        Downstream pressure.
    rho : float
        Downstream density.
    kappa : float
        Isentropic exponent, can be approximated by specific heat ratio.

    Returns
    -------
    p1 : float
        Upstream pressure.
    """

    if debug_print:
        print("orifice_compressible_pressure({}, {}, {}, {}, {}, {}, {})".format(m, do, dp, Cd, p2, rho, kappa))

    approx_pressure = orifice_incompressible_pressure(m, do, dp, Cd, p2, rho)
    adjusted_rho = rho * (approx_pressure / p2)

    if debug_print:
        print("Initial guess: {}, adjusts rho to {}".format(approx_pressure, adjusted_rho))

    approx_m = orifice_compressible(do, dp, Cd, approx_pressure, p2, adjusted_rho, kappa)
    epsilon = abs(m - approx_m) * 1e-5

    if debug_print:
        print("Approx m: {} (vs {}), epslion: {}".format(approx_m, m, epsilon))

    while abs(approx_m - m) > epsilon:
        delta = ((m - approx_m) / m) * approx_pressure * 0.1
        approx_pressure += delta

        approx_m = orifice_compressible(do, dp, Cd, approx_pressure, p2, adjusted_rho, kappa)
        adjusted_rho = rho * (approx_pressure / p2)

        if debug_print:
            print("Approx pressure: {}, mass flow: {} vs {} = {}, delta: {}".format(approx_pressure, approx_m, m, abs(approx_m - m), delta))

    return approx_pressure