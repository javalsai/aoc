import pulp
import numpy as np
from scipy.optimize import minimize


# pretty chatgpt'd
def solve_linear_program(A, B):
    """
    Solve the integer linear programming problem to minimize the sum of variables
    given a matrix A (coefficients) and a vector B (right-hand side).
    """
    # Number of variables (columns in A)
    num_variables = A.shape[1]

    # Define the problem as a minimization problem
    prob = pulp.LpProblem("Minimize_Sum_of_Variables", pulp.LpMinimize)

    # Define variables (all non-negative integers)
    variables = [pulp.LpVariable(f'x{i}', lowBound=0, cat='Integer') for i in range(num_variables)]

    # Objective function: Minimize the sum of the variables
    prob += pulp.lpSum(variables), "Minimize sum"

    # Add the constraints based on A * X = B
    for i in range(A.shape[0]):  # Iterate over each equation (row of A)
        prob += pulp.lpSum(A[i, j] * variables[j] for j in range(num_variables)) == B[i]

    # Solve the problem
    prob.solve(pulp.PULP_CBC_CMD(msg=False))

    # Get the results
    if pulp.LpStatus[prob.status] == 'Optimal':
        solution = [v.varValue for v in variables]
        return solution, sum(solution)
    else:
        return None, None



def parse_buttons(bts):
    return list(map(lambda bt: list(map(int, bt[1:-1].split(","))), bts))

def make_btn(len, bt):
    btn = [0] * len
    for idx in bt:
        btn[idx] = 1
    return btn

def parse_ln(ln):
    ln = ln.strip()
    ln = ln.split(" ")

    coefs = list(map(int, ln[-1][1:-1].split(",")))
    return list(map(lambda bt:
        make_btn(len(coefs), bt)
    , parse_buttons(ln[1:-1]))), coefs


total = 0
with open("input.txt", "r") as file:
    for line in file:
        coefs, terms = parse_ln(line)
        A = np.transpose(np.array(coefs))
        b = np.array(terms)

        solution, min_sum = solve_linear_program(A, b)

        if solution is not None:
            total += min_sum
            # print(f"Optimal solution: {solution}")
            # print(f"Minimum sum: {min_sum}")
        else:
            print("No solution found.")

print(f"total is {total}")
