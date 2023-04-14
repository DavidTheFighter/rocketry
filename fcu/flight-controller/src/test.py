import numpy as np

class KalmanFilter:
    def __init__(self):
        # Initial state estimate (all zeros)
        self.x = np.zeros((6, 1))
        # Initial state covariance (diagonal matrix with large values)
        self.p = np.diag([1e6]*6)
        # Process noise covariance (diagonal matrix with small values)
        self.q = np.diag([1e-6]*6)
        # Measurement noise covariance (diagonal matrix with small values)
        self.r = np.diag([1e-3]*3)

    def update(self, acc, gyro, alt, dt):
        # State transition matrix
        f = np.array([
            [1.0, 0.0, 0.0, dt, 0.0, 0.0],
            [0.0, 1.0, 0.0, 0.0, dt, 0.0],
            [0.0, 0.0, 1.0, 0.0, 0.0, dt],
            [0.0, 0.0, 0.0, 1.0, 0.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 1.0, 0.0],
            [0.0, 0.0, 0.0, 0.0, 0.0, 1.0]
        ])

        # Control input matrix (none in this case)
        b = np.zeros((6, 1))

        # Measurement matrix (only the altitude is measured)
        h = np.array([0.0, 0.0, 0.0, 0.0, 0.0, 1.0]).reshape((1, 6))

        # State estimate update
        x_pred = f @ self.x + b
        p_pred = f @ self.p @ f.T + self.q

        # Measurement update
        y = np.array([0.0, 0.0, alt]).reshape((3, 1)) - h @ x_pred
        s = h @ p_pred @ h.T + self.r
        k = p_pred @ h.T @ np.linalg.inv(s)
        x = x_pred + k @ y
        p = (np.eye(6) - k @ h) @ p_pred

        # Update internal state
        self.x = x
        self.p = p

        return x

def main():
    dt = 0.1  # time step
    dt2 = dt * dt

    states = ['x', 'y', 'z', 'vx', 'vy', 'vz', 'ax', 'ay', 'az', 'qw', 'qx', 'qy', 'qz', 'wx', 'wy', 'wz']
    F = np.eye(len(states))

    w_x = 2
    w_y = 3
    w_z = 4

    # Update position from velocity and acceleration
    F[0, 3] = dt
    F[0, 6] = 0.5 * dt2
    F[1, 4] = dt
    F[1, 7] = 0.5 * dt2
    F[2, 5] = dt
    F[2, 8] = 0.5 * dt2

    # Update velocity from acceleration
    F[3, 6] = dt
    F[4, 7] = dt
    F[5, 8] = dt

    # Update orientation from angular velocity
    F[9:13, 9:13] = -0.5 * dt * np.array([[ 0, -w_x, -w_y, -w_z],
                                    [ w_x,   0,  w_z, -w_y],
                                    [ w_y, -w_z,   0,  w_x],
                                    [ w_z,  w_y, -w_x,   0]])

    print("\t", end='')
    for state in states:
        print(state, end='\t')
    print()

    for i in range(len(states)):
        print("={}".format(states[i]), end="\t")
        for j in range(len(states)):
            print(round(F[i, j], 3), end='\t')
        print()

if __name__ == '__main__':
    main()